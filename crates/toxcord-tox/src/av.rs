//! ToxAV wrapper for audio/video calls.
//!
//! This module provides safe wrappers around `toxav_*` functions for audio/video calls.
//! ToxAV must run on the same thread as the parent Tox instance.

use std::marker::PhantomData;
use std::time::Duration;

use toxcord_tox_sys::*;
use tracing::{debug, info};

use crate::av_callbacks::*;
use crate::av_types::*;
use crate::error::{ToxError, ToxResult};
use crate::tox::ToxInstance;

/// Safe wrapper around a ToxAV instance.
///
/// SAFETY: ToxAvInstance is NOT Send/Sync — it must live on the same thread as
/// the parent ToxInstance. All cross-thread access goes through command channels.
pub struct ToxAvInstance {
    toxav: *mut ToxAV,
    /// Prevent Send/Sync
    _marker: PhantomData<*mut ()>,
}

impl ToxAvInstance {
    /// Create a new ToxAV instance attached to a Tox instance.
    ///
    /// SAFETY: Must be called on the same thread as the Tox instance.
    /// The Tox instance must outlive the ToxAV instance.
    pub fn new(tox: &ToxInstance) -> ToxResult<Self> {
        unsafe {
            let mut err: Toxav_Err_New = 0;
            let toxav = toxav_new(tox.raw(), &mut err);

            if toxav.is_null() {
                let err_msg = match err {
                    1 => "NULL - Tox pointer was null",
                    2 => "MALLOC - Memory allocation failed",
                    3 => "MULTIPLE - ToxAV already exists for this Tox instance",
                    _ => "Unknown error",
                };
                return Err(ToxError::ToxAv(format!("Failed to create ToxAV: {}", err_msg)));
            }

            info!("ToxAV instance created successfully");
            Ok(Self {
                toxav,
                _marker: PhantomData,
            })
        }
    }

    /// Get the raw ToxAV pointer (for FFI calls within the same thread)
    pub fn raw(&self) -> *mut ToxAV {
        self.toxav
    }

    /// Get the iteration interval in milliseconds.
    pub fn iteration_interval(&self) -> Duration {
        unsafe {
            let ms = toxav_iteration_interval(self.toxav);
            Duration::from_millis(ms as u64)
        }
    }

    /// Run one iteration of the ToxAV event loop.
    pub fn iterate(&self) {
        unsafe {
            toxav_iterate(self.toxav);
        }
    }

    /// Register all ToxAV callbacks.
    pub fn register_callbacks(&self) {
        unsafe {
            toxav_callback_call(self.toxav, Some(call_cb), std::ptr::null_mut());
            toxav_callback_call_state(self.toxav, Some(call_state_cb), std::ptr::null_mut());
            toxav_callback_audio_receive_frame(
                self.toxav,
                Some(audio_receive_frame_cb),
                std::ptr::null_mut(),
            );
            toxav_callback_video_receive_frame(
                self.toxav,
                Some(video_receive_frame_cb),
                std::ptr::null_mut(),
            );
            toxav_callback_audio_bit_rate(
                self.toxav,
                Some(audio_bit_rate_cb),
                std::ptr::null_mut(),
            );
            toxav_callback_video_bit_rate(
                self.toxav,
                Some(video_bit_rate_cb),
                std::ptr::null_mut(),
            );
            debug!("ToxAV callbacks registered");
        }
    }

    /// Register callbacks with a user_data pointer.
    /// The user_data should be a `Box<Box<dyn ToxAvEventHandler>>` raw pointer.
    pub fn register_callbacks_with_userdata(&self, user_data: *mut std::ffi::c_void) {
        unsafe {
            toxav_callback_call(self.toxav, Some(call_cb), user_data);
            toxav_callback_call_state(self.toxav, Some(call_state_cb), user_data);
            toxav_callback_audio_receive_frame(
                self.toxav,
                Some(audio_receive_frame_cb),
                user_data,
            );
            toxav_callback_video_receive_frame(
                self.toxav,
                Some(video_receive_frame_cb),
                user_data,
            );
            toxav_callback_audio_bit_rate(self.toxav, Some(audio_bit_rate_cb), user_data);
            toxav_callback_video_bit_rate(self.toxav, Some(video_bit_rate_cb), user_data);
            debug!("ToxAV callbacks registered with userdata");
        }
    }

    // ─── Call Management ───────────────────────────────────────────────────

    /// Initiate an audio/video call with a friend.
    ///
    /// # Arguments
    /// * `friend_number` - The friend to call
    /// * `audio_bit_rate` - Audio bit rate in Kbit/s (6-510, or 0 to disable audio)
    /// * `video_bit_rate` - Video bit rate in Kbit/s (0 to disable video)
    pub fn call(
        &self,
        friend_number: u32,
        audio_bit_rate: u32,
        video_bit_rate: u32,
    ) -> ToxResult<()> {
        unsafe {
            let mut err: Toxav_Err_Call = 0;
            let ok = toxav_call(self.toxav, friend_number, audio_bit_rate, video_bit_rate, &mut err);

            if ok {
                debug!("Call initiated to friend {}", friend_number);
                Ok(())
            } else {
                let err_msg = match err {
                    1 => "MALLOC - Memory allocation failed",
                    2 => "SYNC - Synchronization error",
                    3 => "FRIEND_NOT_FOUND - Friend number invalid",
                    4 => "FRIEND_NOT_CONNECTED - Friend is offline",
                    5 => "FRIEND_ALREADY_IN_CALL - Already in call with this friend",
                    6 => "INVALID_BIT_RATE - Bit rate out of valid range",
                    _ => "Unknown error",
                };
                Err(ToxError::ToxAv(format!("Call failed: {}", err_msg)))
            }
        }
    }

    /// Answer an incoming call from a friend.
    ///
    /// # Arguments
    /// * `friend_number` - The friend whose call to answer
    /// * `audio_bit_rate` - Audio bit rate in Kbit/s (6-510, or 0 to disable audio)
    /// * `video_bit_rate` - Video bit rate in Kbit/s (0 to disable video)
    pub fn answer(
        &self,
        friend_number: u32,
        audio_bit_rate: u32,
        video_bit_rate: u32,
    ) -> ToxResult<()> {
        unsafe {
            let mut err: Toxav_Err_Answer = 0;
            let ok = toxav_answer(self.toxav, friend_number, audio_bit_rate, video_bit_rate, &mut err);

            if ok {
                debug!("Call answered from friend {}", friend_number);
                Ok(())
            } else {
                let err_msg = match err {
                    1 => "SYNC - Synchronization error",
                    2 => "CODEC_INITIALIZATION - Codec initialization failed",
                    3 => "FRIEND_NOT_FOUND - Friend number invalid",
                    4 => "FRIEND_NOT_CALLING - Friend is not calling",
                    5 => "INVALID_BIT_RATE - Bit rate out of valid range",
                    _ => "Unknown error",
                };
                Err(ToxError::ToxAv(format!("Answer failed: {}", err_msg)))
            }
        }
    }

    /// Send a call control command.
    ///
    /// # Arguments
    /// * `friend_number` - The friend for this call
    /// * `control` - The control command to send
    pub fn call_control(&self, friend_number: u32, control: CallControl) -> ToxResult<()> {
        unsafe {
            let mut err: Toxav_Err_Call_Control = 0;
            let ok = toxav_call_control(self.toxav, friend_number, control.to_raw(), &mut err);

            if ok {
                debug!("Call control {:?} sent to friend {}", control, friend_number);
                Ok(())
            } else {
                let err_msg = match err {
                    1 => "SYNC - Synchronization error",
                    2 => "FRIEND_NOT_FOUND - Friend number invalid",
                    3 => "FRIEND_NOT_IN_CALL - Not in call with this friend",
                    4 => "INVALID_TRANSITION - Invalid state transition",
                    _ => "Unknown error",
                };
                Err(ToxError::ToxAv(format!("Call control failed: {}", err_msg)))
            }
        }
    }

    /// Cancel/hangup a call with a friend.
    pub fn hangup(&self, friend_number: u32) -> ToxResult<()> {
        self.call_control(friend_number, CallControl::Cancel)
    }

    /// Mute audio for a call.
    pub fn mute_audio(&self, friend_number: u32) -> ToxResult<()> {
        self.call_control(friend_number, CallControl::MuteAudio)
    }

    /// Unmute audio for a call.
    pub fn unmute_audio(&self, friend_number: u32) -> ToxResult<()> {
        self.call_control(friend_number, CallControl::UnmuteAudio)
    }

    /// Hide video for a call.
    pub fn hide_video(&self, friend_number: u32) -> ToxResult<()> {
        self.call_control(friend_number, CallControl::HideVideo)
    }

    /// Show video for a call.
    pub fn show_video(&self, friend_number: u32) -> ToxResult<()> {
        self.call_control(friend_number, CallControl::ShowVideo)
    }

    // ─── Audio Sending ─────────────────────────────────────────────────────

    /// Send an audio frame to a friend.
    ///
    /// Audio format: PCM samples, interleaved for stereo.
    /// Valid sample rates: 8000, 12000, 16000, 24000, 48000 Hz.
    /// Valid frame durations: 2.5, 5, 10, 20, 40, 60 ms.
    pub fn audio_send_frame(&self, friend_number: u32, frame: &AudioFrame) -> ToxResult<()> {
        if let Err(e) = frame.validate() {
            return Err(ToxError::ToxAv(format!("Invalid audio frame: {}", e)));
        }

        unsafe {
            let mut err: Toxav_Err_Send_Frame = 0;
            let ok = toxav_audio_send_frame(
                self.toxav,
                friend_number,
                frame.pcm.as_ptr(),
                frame.sample_count,
                frame.channels,
                frame.sampling_rate,
                &mut err,
            );

            if ok {
                Ok(())
            } else {
                let err_msg = match err {
                    1 => "NULL - ToxAV pointer was null",
                    2 => "FRIEND_NOT_FOUND - Friend number invalid",
                    3 => "FRIEND_NOT_IN_CALL - Not in call with this friend",
                    4 => "SYNC - Synchronization error",
                    5 => "INVALID - Invalid frame parameters",
                    6 => "PAYLOAD_TYPE_DISABLED - Audio is disabled for this call",
                    7 => "RTP_FAILED - RTP send failed",
                    _ => "Unknown error",
                };
                Err(ToxError::ToxAv(format!("Audio send failed: {}", err_msg)))
            }
        }
    }

    /// Send raw PCM audio to a friend.
    ///
    /// Convenience method that creates an AudioFrame internally.
    pub fn audio_send_raw(
        &self,
        friend_number: u32,
        pcm: &[i16],
        sample_count: usize,
        channels: u8,
        sampling_rate: u32,
    ) -> ToxResult<()> {
        let frame = AudioFrame {
            pcm: pcm.to_vec(),
            sample_count,
            channels,
            sampling_rate,
        };
        self.audio_send_frame(friend_number, &frame)
    }

    /// Set the audio bit rate for a call.
    ///
    /// # Arguments
    /// * `friend_number` - The friend for this call
    /// * `bit_rate` - New bit rate in Kbit/s (6-510)
    pub fn audio_set_bit_rate(&self, friend_number: u32, bit_rate: u32) -> ToxResult<()> {
        unsafe {
            let mut err: Toxav_Err_Bit_Rate_Set = 0;
            let ok = toxav_audio_set_bit_rate(self.toxav, friend_number, bit_rate, &mut err);

            if ok {
                debug!("Audio bit rate set to {} for friend {}", bit_rate, friend_number);
                Ok(())
            } else {
                let err_msg = match err {
                    1 => "SYNC - Synchronization error",
                    2 => "INVALID_BIT_RATE - Bit rate out of valid range",
                    3 => "FRIEND_NOT_FOUND - Friend number invalid",
                    4 => "FRIEND_NOT_IN_CALL - Not in call with this friend",
                    _ => "Unknown error",
                };
                Err(ToxError::ToxAv(format!("Set audio bit rate failed: {}", err_msg)))
            }
        }
    }

    // ─── Video Sending ─────────────────────────────────────────────────────

    /// Send a video frame to a friend.
    ///
    /// Video format: YUV420 planar.
    /// - Y plane: width * height bytes
    /// - U plane: (width/2) * (height/2) bytes
    /// - V plane: (width/2) * (height/2) bytes
    pub fn video_send_frame(&self, friend_number: u32, frame: &VideoFrame) -> ToxResult<()> {
        if let Err(e) = frame.validate() {
            return Err(ToxError::ToxAv(format!("Invalid video frame: {}", e)));
        }

        unsafe {
            let mut err: Toxav_Err_Send_Frame = 0;
            let ok = toxav_video_send_frame(
                self.toxav,
                friend_number,
                frame.width,
                frame.height,
                frame.y.as_ptr(),
                frame.u.as_ptr(),
                frame.v.as_ptr(),
                &mut err,
            );

            if ok {
                Ok(())
            } else {
                let err_msg = match err {
                    1 => "NULL - ToxAV pointer was null",
                    2 => "FRIEND_NOT_FOUND - Friend number invalid",
                    3 => "FRIEND_NOT_IN_CALL - Not in call with this friend",
                    4 => "SYNC - Synchronization error",
                    5 => "INVALID - Invalid frame parameters",
                    6 => "PAYLOAD_TYPE_DISABLED - Video is disabled for this call",
                    7 => "RTP_FAILED - RTP send failed",
                    _ => "Unknown error",
                };
                Err(ToxError::ToxAv(format!("Video send failed: {}", err_msg)))
            }
        }
    }

    /// Set the video bit rate for a call.
    ///
    /// # Arguments
    /// * `friend_number` - The friend for this call
    /// * `bit_rate` - New bit rate in Kbit/s
    pub fn video_set_bit_rate(&self, friend_number: u32, bit_rate: u32) -> ToxResult<()> {
        unsafe {
            let mut err: Toxav_Err_Bit_Rate_Set = 0;
            let ok = toxav_video_set_bit_rate(self.toxav, friend_number, bit_rate, &mut err);

            if ok {
                debug!("Video bit rate set to {} for friend {}", bit_rate, friend_number);
                Ok(())
            } else {
                let err_msg = match err {
                    1 => "SYNC - Synchronization error",
                    2 => "INVALID_BIT_RATE - Bit rate out of valid range",
                    3 => "FRIEND_NOT_FOUND - Friend number invalid",
                    4 => "FRIEND_NOT_IN_CALL - Not in call with this friend",
                    _ => "Unknown error",
                };
                Err(ToxError::ToxAv(format!("Set video bit rate failed: {}", err_msg)))
            }
        }
    }

    // ─── Multi-threaded iteration (optional) ───────────────────────────────

    /// Get audio iteration interval for multi-threaded mode.
    /// Use this if running audio processing in a separate thread.
    pub fn audio_iteration_interval(&self) -> Duration {
        unsafe {
            let ms = toxav_audio_iteration_interval(self.toxav);
            Duration::from_millis(ms as u64)
        }
    }

    /// Run one audio iteration for multi-threaded mode.
    /// Call this in a dedicated audio processing thread.
    pub fn audio_iterate(&self) {
        unsafe {
            toxav_audio_iterate(self.toxav);
        }
    }

    /// Get video iteration interval for multi-threaded mode.
    /// Use this if running video processing in a separate thread.
    pub fn video_iteration_interval(&self) -> Duration {
        unsafe {
            let ms = toxav_video_iteration_interval(self.toxav);
            Duration::from_millis(ms as u64)
        }
    }

    /// Run one video iteration for multi-threaded mode.
    /// Call this in a dedicated video processing thread.
    pub fn video_iterate(&self) {
        unsafe {
            toxav_video_iterate(self.toxav);
        }
    }
}

impl Drop for ToxAvInstance {
    fn drop(&mut self) {
        if !self.toxav.is_null() {
            unsafe {
                toxav_kill(self.toxav);
            }
            info!("ToxAV instance destroyed");
        }
    }
}

// ToxAvInstance is NOT Send/Sync because it contains a raw pointer
// and must be accessed from the same thread as the Tox instance.
// These negative trait impls are automatically inferred from PhantomData<*mut ()>
