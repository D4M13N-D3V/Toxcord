//! Audio/Video call manager.
//!
//! Manages ToxAV call state. Audio capture/playback is managed separately
//! on the tox thread since cpal types are not Send.

use std::collections::HashMap;
use std::sync::Arc;

use tauri::Emitter;
use tracing::{debug, error, info, warn};

use toxcord_tox::{CallStateFlags, ToxAvEventHandler};

use crate::audio::AudioMixer;


/// Call state for a single call
#[derive(Debug, Clone, serde::Serialize)]
pub struct CallState {
    pub friend_number: u32,
    pub state: CallStatus,
    pub has_audio: bool,
    pub has_video: bool,
    pub is_audio_muted: bool,
    pub is_video_muted: bool,
    pub started_at: Option<String>,
}

/// Call status
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CallStatus {
    /// Call is ringing (outgoing)
    RingingOutgoing,
    /// Call is ringing (incoming)
    RingingIncoming,
    /// Call is in progress
    InProgress,
    /// Call has ended
    Ended,
    /// Call failed with error
    Error,
}

/// ToxAV event sent to the frontend
#[derive(Clone, serde::Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ToxAvEvent {
    /// Incoming call from a friend
    IncomingCall {
        friend_number: u32,
        audio_enabled: bool,
        video_enabled: bool,
    },
    /// Call state changed
    CallStateChange {
        friend_number: u32,
        state: String,
        sending_audio: bool,
        sending_video: bool,
        accepting_audio: bool,
        accepting_video: bool,
    },
    /// Call ended
    CallEnded {
        friend_number: u32,
        reason: String,
    },
    /// Audio level update for a peer
    AudioLevelUpdate {
        friend_number: u32,
        level: f32,
    },
}

/// Manages active call state.
/// Note: Audio capture/playback is managed on the tox thread,
/// not here, because cpal types are not Send.
#[derive(Default)]
pub struct AvManager {
    /// Active calls keyed by friend_number
    calls: HashMap<u32, CallState>,
    /// Whether audio is globally muted
    is_muted: bool,
    /// Whether audio is globally deafened
    is_deafened: bool,
}

impl AvManager {
    pub fn new() -> Self {
        Self {
            calls: HashMap::new(),
            is_muted: false,
            is_deafened: false,
        }
    }

    /// Start a call with a friend
    pub fn start_call(&mut self, friend_number: u32, with_video: bool) {
        let call = CallState {
            friend_number,
            state: CallStatus::RingingOutgoing,
            has_audio: true,
            has_video: with_video,
            is_audio_muted: false,
            is_video_muted: !with_video,
            started_at: None,
        };
        self.calls.insert(friend_number, call);
        info!("Started call with friend {}", friend_number);
    }

    /// Handle an incoming call
    pub fn handle_incoming_call(&mut self, friend_number: u32, audio_enabled: bool, video_enabled: bool) {
        let call = CallState {
            friend_number,
            state: CallStatus::RingingIncoming,
            has_audio: audio_enabled,
            has_video: video_enabled,
            is_audio_muted: false,
            is_video_muted: !video_enabled,
            started_at: None,
        };
        self.calls.insert(friend_number, call);
        info!("Incoming call from friend {} (audio: {}, video: {})",
              friend_number, audio_enabled, video_enabled);
    }

    /// Update call state based on ToxAV callback
    pub fn update_call_state(&mut self, friend_number: u32, state: CallStateFlags) {
        if let Some(call) = self.calls.get_mut(&friend_number) {
            let old_state = call.state;
            if state.error || state.finished {
                call.state = if state.error {
                    CallStatus::Error
                } else {
                    CallStatus::Ended
                };
            } else if state.is_active() {
                if call.state != CallStatus::InProgress {
                    call.state = CallStatus::InProgress;
                    call.started_at = Some(chrono::Utc::now().to_rfc3339());
                    info!("Call with friend {} transitioned from {:?} to InProgress", friend_number, old_state);
                }
            }

            call.has_audio = state.has_audio();
            call.has_video = state.has_video();
            debug!("Call state for friend {} updated: {:?} -> {:?}, has_audio={}, has_video={}",
                   friend_number, old_state, call.state, call.has_audio, call.has_video);
        } else {
            warn!("update_call_state called for friend {} but no call exists", friend_number);
        }
    }

    /// End a call
    pub fn end_call(&mut self, friend_number: u32) {
        if let Some(call) = self.calls.get_mut(&friend_number) {
            call.state = CallStatus::Ended;
        }
        self.calls.remove(&friend_number);
        info!("Ended call with friend {}", friend_number);
    }

    /// Get call state for a friend
    pub fn get_call(&self, friend_number: u32) -> Option<&CallState> {
        self.calls.get(&friend_number)
    }

    /// Get all active calls
    pub fn get_all_calls(&self) -> Vec<&CallState> {
        self.calls.values().collect()
    }

    /// Check if there's an active call with a friend
    pub fn has_call(&self, friend_number: u32) -> bool {
        self.calls.contains_key(&friend_number)
    }

    /// Check if there's any active call
    pub fn has_any_call(&self) -> bool {
        !self.calls.is_empty()
    }

    /// Set mute state
    pub fn set_muted(&mut self, muted: bool) {
        self.is_muted = muted;
        debug!("Audio muted: {}", muted);
    }

    /// Set deafen state
    pub fn set_deafened(&mut self, deafened: bool) {
        self.is_deafened = deafened;
        debug!("Audio deafened: {}", deafened);
    }

    /// Check if muted
    pub fn is_muted(&self) -> bool {
        self.is_muted
    }

    /// Check if deafened
    pub fn is_deafened(&self) -> bool {
        self.is_deafened
    }
}

/// ToxAV event handler that forwards events to the frontend via Tauri
/// and pushes received audio to the mixer for playback
pub struct TauriAvEventHandler {
    app_handle: tauri::AppHandle,
    av_manager: Arc<std::sync::Mutex<AvManager>>,
    /// Mixer for combining audio from multiple sources
    mixer: Arc<std::sync::Mutex<AudioMixer>>,
}

impl TauriAvEventHandler {
    pub fn new(
        app_handle: tauri::AppHandle,
        av_manager: Arc<std::sync::Mutex<AvManager>>,
        mixer: Arc<std::sync::Mutex<AudioMixer>>,
    ) -> Self {
        Self {
            app_handle,
            av_manager,
            mixer,
        }
    }

    fn emit(&self, event: ToxAvEvent) {
        if let Err(e) = self.app_handle.emit("toxav://event", &event) {
            error!("Failed to emit ToxAV event: {e}");
        }
    }
}

impl ToxAvEventHandler for TauriAvEventHandler {
    fn on_call(&self, friend_number: u32, audio_enabled: bool, video_enabled: bool) {
        info!("Incoming call from friend {}", friend_number);

        // Update manager state synchronously using blocking lock
        if let Ok(mut mgr) = self.av_manager.lock() {
            mgr.handle_incoming_call(friend_number, audio_enabled, video_enabled);
        }

        self.emit(ToxAvEvent::IncomingCall {
            friend_number,
            audio_enabled,
            video_enabled,
        });
    }

    fn on_call_state(&self, friend_number: u32, state: CallStateFlags) {
        info!("Call state change for friend {}: {:?} (error={}, finished={}, is_active={})",
              friend_number, state, state.error, state.finished, state.is_active());

        let state_str = if state.error {
            "error"
        } else if state.finished {
            "ended"
        } else if state.is_active() {
            "in_progress"
        } else {
            "unknown"
        };

        // Update manager state
        if let Ok(mut mgr) = self.av_manager.lock() {
            mgr.update_call_state(friend_number, state);
        }

        self.emit(ToxAvEvent::CallStateChange {
            friend_number,
            state: state_str.to_string(),
            sending_audio: state.sending_audio,
            sending_video: state.sending_video,
            accepting_audio: state.accepting_audio,
            accepting_video: state.accepting_video,
        });

        // If call ended, emit end event and clean up mixer
        if state.finished || state.error {
            let reason = if state.error { "error" } else { "hangup" };
            self.emit(ToxAvEvent::CallEnded {
                friend_number,
                reason: reason.to_string(),
            });

            // Remove this friend's audio source from the mixer
            if let Ok(mut mixer) = self.mixer.lock() {
                mixer.remove_source(friend_number);
            }
        }
    }

    fn on_audio_receive_frame(
        &self,
        friend_number: u32,
        pcm: &[i16],
        sample_count: usize,
        channels: u8,
        sampling_rate: u32,
    ) {
        // Log received audio for debugging
        debug!(
            "Received audio frame from friend {}: {} samples, {} channels, {} Hz, pcm_len={}",
            friend_number, sample_count, channels, sampling_rate, pcm.len()
        );

        // Push received audio to the mixer for playback
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.push_frame(friend_number, pcm.to_vec());
            debug!("Pushed {} samples to mixer for friend {}", pcm.len(), friend_number);
        }
    }

    fn on_video_receive_frame(
        &self,
        friend_number: u32,
        width: u16,
        height: u16,
        _y: &[u8],
        _u: &[u8],
        _v: &[u8],
        _y_stride: i32,
        _u_stride: i32,
        _v_stride: i32,
    ) {
        // TODO: Phase 5.6 - Video support
        debug!(
            "Received video frame from friend {}: {}x{}",
            friend_number, width, height
        );
    }

    fn on_audio_bit_rate(&self, friend_number: u32, audio_bit_rate: u32) {
        debug!(
            "Audio bit rate changed for friend {}: {} kbit/s",
            friend_number, audio_bit_rate
        );
    }

    fn on_video_bit_rate(&self, friend_number: u32, video_bit_rate: u32) {
        debug!(
            "Video bit rate changed for friend {}: {} kbit/s",
            friend_number, video_bit_rate
        );
    }
}
