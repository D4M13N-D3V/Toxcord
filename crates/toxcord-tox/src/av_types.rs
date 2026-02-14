//! ToxAV types for audio/video calls.

/// Call state flags returned by ToxAV callbacks.
/// These are bitmask flags from TOXAV_FRIEND_CALL_STATE_*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CallStateFlags {
    /// Call encountered an error
    pub error: bool,
    /// Call has finished (normally or abnormally)
    pub finished: bool,
    /// Friend is sending audio
    pub sending_audio: bool,
    /// Friend is sending video
    pub sending_video: bool,
    /// Friend is accepting audio
    pub accepting_audio: bool,
    /// Friend is accepting video
    pub accepting_video: bool,
}

impl CallStateFlags {
    /// Parse from raw bitmask value
    pub fn from_raw(state: u32) -> Self {
        Self {
            error: (state & 1) != 0,      // TOXAV_FRIEND_CALL_STATE_ERROR
            finished: (state & 2) != 0,   // TOXAV_FRIEND_CALL_STATE_FINISHED
            sending_audio: (state & 4) != 0,   // TOXAV_FRIEND_CALL_STATE_SENDING_A
            sending_video: (state & 8) != 0,   // TOXAV_FRIEND_CALL_STATE_SENDING_V
            accepting_audio: (state & 16) != 0, // TOXAV_FRIEND_CALL_STATE_ACCEPTING_A
            accepting_video: (state & 32) != 0, // TOXAV_FRIEND_CALL_STATE_ACCEPTING_V
        }
    }

    /// Check if the call is active (not finished and no error)
    pub fn is_active(&self) -> bool {
        !self.finished && !self.error
    }

    /// Check if the call has audio capability
    pub fn has_audio(&self) -> bool {
        self.sending_audio || self.accepting_audio
    }

    /// Check if the call has video capability
    pub fn has_video(&self) -> bool {
        self.sending_video || self.accepting_video
    }
}

/// Call control commands for ToxAV.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallControl {
    /// Resume a paused call
    Resume = 0,
    /// Pause the call (stop sending but keep receiving)
    Pause = 1,
    /// Cancel/end the call
    Cancel = 2,
    /// Mute audio (stop sending audio)
    MuteAudio = 3,
    /// Unmute audio (resume sending audio)
    UnmuteAudio = 4,
    /// Hide video (stop sending video)
    HideVideo = 5,
    /// Show video (resume sending video)
    ShowVideo = 6,
}

impl CallControl {
    /// Convert to raw TOXAV_CALL_CONTROL_* value
    pub fn to_raw(self) -> u32 {
        self as u32
    }
}

/// Audio frame for sending/receiving via ToxAV.
///
/// Audio format:
/// - PCM samples, interleaved for stereo: [L0, R0, L1, R1, ...]
/// - Valid sample rates: 8000, 12000, 16000, 24000, 48000 Hz
/// - Valid channels: 1 (mono) or 2 (stereo)
/// - Valid frame durations: 2.5, 5, 10, 20, 40, 60 ms
#[derive(Debug, Clone)]
pub struct AudioFrame {
    /// PCM samples (i16)
    pub pcm: Vec<i16>,
    /// Number of samples per channel
    pub sample_count: usize,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u8,
    /// Sampling rate in Hz
    pub sampling_rate: u32,
}

impl AudioFrame {
    /// Create a new audio frame
    pub fn new(pcm: Vec<i16>, sample_count: usize, channels: u8, sampling_rate: u32) -> Self {
        Self {
            pcm,
            sample_count,
            channels,
            sampling_rate,
        }
    }

    /// Create a frame from raw PCM data
    pub fn from_pcm(pcm: Vec<i16>, channels: u8, sampling_rate: u32) -> Self {
        let sample_count = pcm.len() / channels as usize;
        Self {
            pcm,
            sample_count,
            channels,
            sampling_rate,
        }
    }

    /// Get duration of frame in milliseconds
    pub fn duration_ms(&self) -> f64 {
        (self.sample_count as f64 / self.sampling_rate as f64) * 1000.0
    }

    /// Validate that the frame has valid parameters for ToxAV
    pub fn validate(&self) -> Result<(), &'static str> {
        // Check sample rate
        if ![8000, 12000, 16000, 24000, 48000].contains(&self.sampling_rate) {
            return Err("Invalid sample rate. Must be 8000, 12000, 16000, 24000, or 48000 Hz");
        }

        // Check channels
        if self.channels != 1 && self.channels != 2 {
            return Err("Invalid channel count. Must be 1 (mono) or 2 (stereo)");
        }

        // Check that PCM length matches sample_count * channels
        let expected_len = self.sample_count * self.channels as usize;
        if self.pcm.len() != expected_len {
            return Err("PCM data length doesn't match sample_count * channels");
        }

        Ok(())
    }
}

/// Video frame in YUV420 planar format for ToxAV.
///
/// YUV420 format:
/// - Y plane: width * height bytes (luminance)
/// - U plane: (width/2) * (height/2) bytes (chroma)
/// - V plane: (width/2) * (height/2) bytes (chroma)
#[derive(Debug, Clone)]
pub struct VideoFrame {
    /// Y (luminance) plane
    pub y: Vec<u8>,
    /// U (chroma) plane
    pub u: Vec<u8>,
    /// V (chroma) plane
    pub v: Vec<u8>,
    /// Frame width in pixels
    pub width: u16,
    /// Frame height in pixels
    pub height: u16,
}

impl VideoFrame {
    /// Create a new video frame
    pub fn new(y: Vec<u8>, u: Vec<u8>, v: Vec<u8>, width: u16, height: u16) -> Self {
        Self { y, u, v, width, height }
    }

    /// Get expected Y plane size
    pub fn y_plane_size(width: u16, height: u16) -> usize {
        width as usize * height as usize
    }

    /// Get expected U/V plane size
    pub fn uv_plane_size(width: u16, height: u16) -> usize {
        (width as usize / 2) * (height as usize / 2)
    }

    /// Validate that the frame has correct plane sizes
    pub fn validate(&self) -> Result<(), &'static str> {
        let y_size = Self::y_plane_size(self.width, self.height);
        let uv_size = Self::uv_plane_size(self.width, self.height);

        if self.y.len() != y_size {
            return Err("Y plane size doesn't match width * height");
        }
        if self.u.len() != uv_size {
            return Err("U plane size doesn't match (width/2) * (height/2)");
        }
        if self.v.len() != uv_size {
            return Err("V plane size doesn't match (width/2) * (height/2)");
        }

        Ok(())
    }
}

/// Received video frame with stride information.
/// ToxAV may provide frames with padding/stride that differs from width.
#[derive(Debug, Clone)]
pub struct VideoFrameWithStride {
    /// Y (luminance) plane data
    pub y: Vec<u8>,
    /// U (chroma) plane data
    pub u: Vec<u8>,
    /// V (chroma) plane data
    pub v: Vec<u8>,
    /// Frame width in pixels
    pub width: u16,
    /// Frame height in pixels
    pub height: u16,
    /// Y plane stride (bytes per row, may be >= width)
    pub y_stride: i32,
    /// U plane stride
    pub u_stride: i32,
    /// V plane stride
    pub v_stride: i32,
}

impl VideoFrameWithStride {
    /// Convert to a VideoFrame without stride (copies data)
    pub fn to_video_frame(&self) -> VideoFrame {
        let y_stride_abs = self.y_stride.unsigned_abs() as usize;
        let u_stride_abs = self.u_stride.unsigned_abs() as usize;
        let v_stride_abs = self.v_stride.unsigned_abs() as usize;
        let w = self.width as usize;
        let h = self.height as usize;
        let uv_h = h / 2;
        let uv_w = w / 2;

        // Copy Y plane, removing stride padding
        let mut y = Vec::with_capacity(w * h);
        for row in 0..h {
            let start = row * y_stride_abs;
            y.extend_from_slice(&self.y[start..start + w]);
        }

        // Copy U plane
        let mut u = Vec::with_capacity(uv_w * uv_h);
        for row in 0..uv_h {
            let start = row * u_stride_abs;
            u.extend_from_slice(&self.u[start..start + uv_w]);
        }

        // Copy V plane
        let mut v = Vec::with_capacity(uv_w * uv_h);
        for row in 0..uv_h {
            let start = row * v_stride_abs;
            v.extend_from_slice(&self.v[start..start + uv_w]);
        }

        VideoFrame {
            y,
            u,
            v,
            width: self.width,
            height: self.height,
        }
    }
}

/// Audio/video bit rate settings
#[derive(Debug, Clone, Copy)]
pub struct BitRateSettings {
    /// Audio bit rate in Kbit/s (6-510, or 0 to disable)
    pub audio_bit_rate: u32,
    /// Video bit rate in Kbit/s (0 to disable)
    pub video_bit_rate: u32,
}

impl Default for BitRateSettings {
    fn default() -> Self {
        Self {
            audio_bit_rate: 48,  // Good quality voice
            video_bit_rate: 0,   // No video by default
        }
    }
}

impl BitRateSettings {
    /// Voice-only call settings
    pub fn voice_only() -> Self {
        Self {
            audio_bit_rate: 48,
            video_bit_rate: 0,
        }
    }

    /// Video call settings
    pub fn video_call() -> Self {
        Self {
            audio_bit_rate: 48,
            video_bit_rate: 1000, // ~1 Mbit/s for video
        }
    }

    /// High quality video call
    pub fn high_quality() -> Self {
        Self {
            audio_bit_rate: 64,
            video_bit_rate: 2500,
        }
    }
}
