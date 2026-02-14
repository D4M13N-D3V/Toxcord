//! ToxAV callback trampolines and event handler trait.

use crate::av_types::CallStateFlags;

/// Trait for handling ToxAV events. Implement this to receive audio/video callbacks.
pub trait ToxAvEventHandler: Send + 'static {
    /// Called when an incoming call is received from a friend.
    ///
    /// # Arguments
    /// * `friend_number` - The friend who is calling
    /// * `audio_enabled` - Whether audio is enabled for the call
    /// * `video_enabled` - Whether video is enabled for the call
    fn on_call(&self, friend_number: u32, audio_enabled: bool, video_enabled: bool);

    /// Called when the call state changes.
    ///
    /// # Arguments
    /// * `friend_number` - The friend whose call state changed
    /// * `state` - The new call state flags
    fn on_call_state(&self, friend_number: u32, state: CallStateFlags);

    /// Called when an audio frame is received from a friend.
    ///
    /// # Arguments
    /// * `friend_number` - The friend who sent the audio
    /// * `pcm` - The PCM audio samples
    /// * `sample_count` - Number of samples per channel
    /// * `channels` - Number of audio channels (1 = mono, 2 = stereo)
    /// * `sampling_rate` - Sample rate in Hz (8000, 12000, 16000, 24000, or 48000)
    fn on_audio_receive_frame(
        &self,
        friend_number: u32,
        pcm: &[i16],
        sample_count: usize,
        channels: u8,
        sampling_rate: u32,
    );

    /// Called when a video frame is received from a friend.
    ///
    /// # Arguments
    /// * `friend_number` - The friend who sent the video
    /// * `width` - Frame width in pixels
    /// * `height` - Frame height in pixels
    /// * `y` - Y plane data (luminance)
    /// * `u` - U plane data (chroma)
    /// * `v` - V plane data (chroma)
    /// * `y_stride` - Y plane stride (bytes per row)
    /// * `u_stride` - U plane stride
    /// * `v_stride` - V plane stride
    fn on_video_receive_frame(
        &self,
        friend_number: u32,
        width: u16,
        height: u16,
        y: &[u8],
        u: &[u8],
        v: &[u8],
        y_stride: i32,
        u_stride: i32,
        v_stride: i32,
    );

    /// Called when the audio bit rate changes (feedback from encoder).
    ///
    /// # Arguments
    /// * `friend_number` - The friend for this call
    /// * `audio_bit_rate` - New audio bit rate in Kbit/s
    fn on_audio_bit_rate(&self, friend_number: u32, audio_bit_rate: u32);

    /// Called when the video bit rate changes (feedback from encoder).
    ///
    /// # Arguments
    /// * `friend_number` - The friend for this call
    /// * `video_bit_rate` - New video bit rate in Kbit/s
    fn on_video_bit_rate(&self, friend_number: u32, video_bit_rate: u32);
}

// ─── extern "C" callback trampolines ───────────────────────────────────────

/// The user_data pointer passed to all ToxAV callbacks is a raw pointer to a
/// `Box<dyn ToxAvEventHandler>`. These trampolines extract it and dispatch.

macro_rules! extract_av_handler {
    ($user_data:expr) => {{
        let handler = &*($user_data as *const Box<dyn ToxAvEventHandler>);
        handler.as_ref()
    }};
}

/// Callback for incoming calls
pub unsafe extern "C" fn call_cb(
    _toxav: *mut toxcord_tox_sys::ToxAV,
    friend_number: u32,
    audio_enabled: bool,
    video_enabled: bool,
    user_data: *mut std::ffi::c_void,
) {
    if user_data.is_null() {
        return;
    }
    let handler = extract_av_handler!(user_data);
    handler.on_call(friend_number, audio_enabled, video_enabled);
}

/// Callback for call state changes
pub unsafe extern "C" fn call_state_cb(
    _toxav: *mut toxcord_tox_sys::ToxAV,
    friend_number: u32,
    state: u32,
    user_data: *mut std::ffi::c_void,
) {
    if user_data.is_null() {
        return;
    }
    let handler = extract_av_handler!(user_data);
    let flags = CallStateFlags::from_raw(state);
    handler.on_call_state(friend_number, flags);
}

/// Callback for receiving audio frames
pub unsafe extern "C" fn audio_receive_frame_cb(
    _toxav: *mut toxcord_tox_sys::ToxAV,
    friend_number: u32,
    pcm: *const i16,
    sample_count: usize,
    channels: u8,
    sampling_rate: u32,
    user_data: *mut std::ffi::c_void,
) {
    if user_data.is_null() || pcm.is_null() {
        return;
    }
    let handler = extract_av_handler!(user_data);
    let pcm_slice = std::slice::from_raw_parts(pcm, sample_count * channels as usize);
    handler.on_audio_receive_frame(friend_number, pcm_slice, sample_count, channels, sampling_rate);
}

/// Callback for receiving video frames
pub unsafe extern "C" fn video_receive_frame_cb(
    _toxav: *mut toxcord_tox_sys::ToxAV,
    friend_number: u32,
    width: u16,
    height: u16,
    y: *const u8,
    u: *const u8,
    v: *const u8,
    y_stride: i32,
    u_stride: i32,
    v_stride: i32,
    user_data: *mut std::ffi::c_void,
) {
    if user_data.is_null() || y.is_null() || u.is_null() || v.is_null() {
        return;
    }
    let handler = extract_av_handler!(user_data);

    // Calculate plane sizes based on stride
    let y_stride_abs = y_stride.unsigned_abs() as usize;
    let u_stride_abs = u_stride.unsigned_abs() as usize;
    let v_stride_abs = v_stride.unsigned_abs() as usize;
    let h = height as usize;
    let uv_h = h / 2;

    let y_size = y_stride_abs * h;
    let u_size = u_stride_abs * uv_h;
    let v_size = v_stride_abs * uv_h;

    let y_slice = std::slice::from_raw_parts(y, y_size);
    let u_slice = std::slice::from_raw_parts(u, u_size);
    let v_slice = std::slice::from_raw_parts(v, v_size);

    handler.on_video_receive_frame(
        friend_number,
        width,
        height,
        y_slice,
        u_slice,
        v_slice,
        y_stride,
        u_stride,
        v_stride,
    );
}

/// Callback for audio bit rate changes
pub unsafe extern "C" fn audio_bit_rate_cb(
    _toxav: *mut toxcord_tox_sys::ToxAV,
    friend_number: u32,
    audio_bit_rate: u32,
    user_data: *mut std::ffi::c_void,
) {
    if user_data.is_null() {
        return;
    }
    let handler = extract_av_handler!(user_data);
    handler.on_audio_bit_rate(friend_number, audio_bit_rate);
}

/// Callback for video bit rate changes
pub unsafe extern "C" fn video_bit_rate_cb(
    _toxav: *mut toxcord_tox_sys::ToxAV,
    friend_number: u32,
    video_bit_rate: u32,
    user_data: *mut std::ffi::c_void,
) {
    if user_data.is_null() {
        return;
    }
    let handler = extract_av_handler!(user_data);
    handler.on_video_bit_rate(friend_number, video_bit_rate);
}
