//! Tauri commands for audio/video calls.

use tauri::State;

use crate::audio::{AudioCapture, AudioDevice, AudioPlayback};
use crate::managers::av_manager::CallState;
use crate::AppState;

/// Start a call with a friend
#[tauri::command]
pub async fn call_friend(
    state: State<'_, AppState>,
    friend_number: u32,
    with_video: bool,
) -> Result<(), String> {
    // Get the ToxAV manager and initiate call
    let tox_guard = state.tox_manager.lock().await;
    let tox = tox_guard.as_ref().ok_or("Not logged in")?;

    let mgr = tox.lock().await;
    mgr.call(friend_number, with_video).await?;

    Ok(())
}

/// Answer an incoming call
#[tauri::command]
pub async fn answer_call(
    state: State<'_, AppState>,
    friend_number: u32,
    with_video: bool,
) -> Result<(), String> {
    let tox_guard = state.tox_manager.lock().await;
    let tox = tox_guard.as_ref().ok_or("Not logged in")?;

    let mgr = tox.lock().await;
    mgr.answer(friend_number, with_video).await?;

    Ok(())
}

/// Hangup/reject a call
#[tauri::command]
pub async fn hangup_call(
    state: State<'_, AppState>,
    friend_number: u32,
) -> Result<(), String> {
    let tox_guard = state.tox_manager.lock().await;
    let tox = tox_guard.as_ref().ok_or("Not logged in")?;

    let mgr = tox.lock().await;
    mgr.hangup(friend_number).await?;

    Ok(())
}

/// Toggle audio mute for a call
#[tauri::command]
pub async fn toggle_mute(
    state: State<'_, AppState>,
    friend_number: u32,
    muted: bool,
) -> Result<(), String> {
    let tox_guard = state.tox_manager.lock().await;
    let tox = tox_guard.as_ref().ok_or("Not logged in")?;

    let mgr = tox.lock().await;
    if muted {
        mgr.mute_audio(friend_number).await?;
    } else {
        mgr.unmute_audio(friend_number).await?;
    }

    Ok(())
}

/// Toggle video for a call
#[tauri::command]
pub async fn toggle_video(
    state: State<'_, AppState>,
    friend_number: u32,
    enabled: bool,
) -> Result<(), String> {
    let tox_guard = state.tox_manager.lock().await;
    let tox = tox_guard.as_ref().ok_or("Not logged in")?;

    let mgr = tox.lock().await;
    if enabled {
        mgr.show_video(friend_number).await?;
    } else {
        mgr.hide_video(friend_number).await?;
    }

    Ok(())
}

/// Get current call state
#[tauri::command]
pub async fn get_call_state(
    state: State<'_, AppState>,
    friend_number: u32,
) -> Result<Option<CallState>, String> {
    let tox_guard = state.tox_manager.lock().await;
    let tox = tox_guard.as_ref().ok_or("Not logged in")?;

    let mgr = tox.lock().await;
    Ok(mgr.get_call_state(friend_number).await)
}

/// List available audio input devices
#[tauri::command]
pub fn list_audio_input_devices() -> Result<Vec<AudioDevice>, String> {
    AudioCapture::list_devices().map_err(|e| e.to_string())
}

/// List available audio output devices
#[tauri::command]
pub fn list_audio_output_devices() -> Result<Vec<AudioDevice>, String> {
    AudioPlayback::list_devices().map_err(|e| e.to_string())
}
