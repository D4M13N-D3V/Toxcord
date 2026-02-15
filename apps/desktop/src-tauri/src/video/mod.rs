//! Video capture module for ToxAV.
//!
//! This module provides:
//! - Video capture from camera (via nokhwa)
//! - RGB to YUV420 conversion for ToxAV
//! - Frame transport to frontend

pub mod capture;
pub mod convert;

pub use capture::{VideoCapture, VideoCaptureError, VideoFrameData};

/// Default video configuration
pub const DEFAULT_VIDEO_WIDTH: u32 = 640;
pub const DEFAULT_VIDEO_HEIGHT: u32 = 480;
pub const DEFAULT_VIDEO_FPS: u32 = 15;

/// Video device information
#[derive(Debug, Clone, serde::Serialize)]
pub struct VideoDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

/// Video error type
#[derive(Debug, thiserror::Error)]
pub enum VideoError {
    #[error("Failed to initialize video: {0}")]
    Init(String),

    #[error("Camera not found: {0}")]
    CameraNotFound(String),

    #[error("Failed to capture frame: {0}")]
    Capture(String),

    #[error("Conversion error: {0}")]
    Conversion(String),

    #[error("Channel send error")]
    ChannelSend,
}

pub type VideoResult<T> = Result<T, VideoError>;
