//! Audio capture and playback module for ToxAV.
//!
//! This module provides:
//! - Audio capture from microphone (via cpal)
//! - Audio playback to speakers (via cpal)
//! - Audio mixing for voice channels (multiple simultaneous streams)
//! - Resampling to/from ToxAV's required formats

pub mod capture;
pub mod mixer;
pub mod playback;

pub use capture::AudioCapture;
pub use mixer::AudioMixer;
pub use playback::AudioPlayback;

/// Standard ToxAV audio configuration
pub const TOXAV_SAMPLE_RATE: u32 = 48000;
pub const TOXAV_CHANNELS: u8 = 1; // Mono for voice
pub const TOXAV_FRAME_DURATION_MS: u32 = 20; // 20ms frames
pub const TOXAV_SAMPLES_PER_FRAME: usize = (TOXAV_SAMPLE_RATE * TOXAV_FRAME_DURATION_MS / 1000) as usize;

/// Audio device information
#[derive(Debug, Clone, serde::Serialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

/// Audio error type
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("Failed to initialize audio: {0}")]
    Init(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Failed to build stream: {0}")]
    StreamBuild(String),

    #[error("Stream error: {0}")]
    Stream(String),

    #[error("Resampling error: {0}")]
    Resample(String),

    #[error("Channel send error")]
    ChannelSend,

    #[error("Channel receive error")]
    ChannelRecv,
}

pub type AudioResult<T> = Result<T, AudioError>;
