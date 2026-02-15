//! Tauri commands for audio/video calls.

use tauri::State;

use crate::audio::{AudioCapture, AudioDevice, AudioPlayback};
use crate::managers::av_manager::CallState;
use crate::video::{ScreenCapture, ScreenInfo, VideoCapture, VideoDevice};
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

/// List available video input devices (cameras)
#[tauri::command]
pub fn list_video_devices() -> Result<Vec<VideoDevice>, String> {
    VideoCapture::list_devices().map_err(|e| e.to_string())
}

/// Set the selected microphone device
#[tauri::command]
pub async fn set_audio_input_device(
    state: State<'_, AppState>,
    device_id: String,
) -> Result<(), String> {
    let index = device_id.parse::<u32>().ok();
    *state.selected_mic_index.lock().await = index;
    tracing::info!("Selected microphone device index: {:?}", index);
    Ok(())
}

/// Set the selected speaker device
#[tauri::command]
pub async fn set_audio_output_device(
    state: State<'_, AppState>,
    device_id: String,
) -> Result<(), String> {
    let index = device_id.parse::<u32>().ok();
    *state.selected_speaker_index.lock().await = index;
    tracing::info!("Selected speaker device index: {:?}", index);
    Ok(())
}

/// Set the selected camera device
#[tauri::command]
pub async fn set_video_device(
    state: State<'_, AppState>,
    device_id: String,
) -> Result<(), String> {
    let index = device_id.parse::<u32>().ok();
    *state.selected_camera_index.lock().await = index;
    tracing::info!("Selected camera device index: {:?}", index);
    Ok(())
}

/// Camera status for diagnostics
#[derive(serde::Serialize)]
pub struct CameraStatus {
    pub has_usb_camera: bool,
    pub has_video_device: bool,
    pub needs_driver_load: bool,
    pub usb_camera_name: Option<String>,
}

/// Check camera status - detect if USB camera exists but driver not loaded
#[tauri::command]
pub fn check_camera_status() -> CameraStatus {
    #[cfg(target_os = "linux")]
    {
        // Check for USB cameras by scanning /sys/bus/usb/devices
        let mut has_usb_camera = false;
        let mut usb_camera_name = None;

        if let Ok(entries) = std::fs::read_dir("/sys/bus/usb/devices") {
            for entry in entries.flatten() {
                let path = entry.path();
                // Check if this USB device is a video class device (0x0e)
                let class_path = path.join("bInterfaceClass");
                if let Ok(class) = std::fs::read_to_string(&class_path) {
                    if class.trim() == "0e" {
                        has_usb_camera = true;
                        // Try to get product name
                        let product_path = path.join("../product");
                        if let Ok(name) = std::fs::read_to_string(product_path) {
                            usb_camera_name = Some(name.trim().to_string());
                        }
                        break;
                    }
                }
                // Also check parent device class
                let parent_class = path.join("bDeviceClass");
                if let Ok(class) = std::fs::read_to_string(&parent_class) {
                    if class.trim() == "ef" {
                        // Miscellaneous class, could be camera
                        let product_path = path.join("product");
                        if let Ok(name) = std::fs::read_to_string(&product_path) {
                            let name_lower = name.to_lowercase();
                            if name_lower.contains("cam") || name_lower.contains("webcam") || name_lower.contains("video") {
                                has_usb_camera = true;
                                usb_camera_name = Some(name.trim().to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Check for /dev/video* devices
        let has_video_device = (0..10).any(|i| {
            std::path::Path::new(&format!("/dev/video{}", i)).exists()
        });

        let needs_driver_load = has_usb_camera && !has_video_device;

        tracing::info!(
            "Camera status: usb_camera={}, video_device={}, needs_driver={}",
            has_usb_camera, has_video_device, needs_driver_load
        );

        CameraStatus {
            has_usb_camera,
            has_video_device,
            needs_driver_load,
            usb_camera_name,
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        CameraStatus {
            has_usb_camera: false,
            has_video_device: false,
            needs_driver_load: false,
            usb_camera_name: None,
        }
    }
}

/// Try to load the UVC video driver (requires pkexec for graphical sudo)
#[tauri::command]
pub async fn load_camera_driver() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        tracing::info!("Attempting to load uvcvideo driver via pkexec");

        let output = Command::new("pkexec")
            .args(["modprobe", "uvcvideo"])
            .output()
            .map_err(|e| format!("Failed to run pkexec: {}", e))?;

        if output.status.success() {
            tracing::info!("Successfully loaded uvcvideo driver");
            // Give the system a moment to create the device nodes
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to load driver: {}", stderr);
            Err(format!("Failed to load camera driver: {}", stderr))
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err("Driver loading only supported on Linux".to_string())
    }
}

// ─── Screen Sharing ───────────────────────────────────────────────────────

/// List available screens for sharing
#[tauri::command]
pub fn list_screens() -> Result<Vec<ScreenInfo>, String> {
    ScreenCapture::list_screens().map_err(|e| e.to_string())
}

/// Start screen sharing (replaces camera capture)
#[tauri::command]
pub async fn start_screen_share(
    state: State<'_, AppState>,
    screen_id: Option<u32>,
) -> Result<(), String> {
    tracing::info!("Starting screen share with screen_id: {:?}", screen_id);
    *state.screen_share_id.lock().await = screen_id;
    *state.is_screen_sharing.lock().await = true;
    Ok(())
}

/// Stop screen sharing (switch back to camera)
#[tauri::command]
pub async fn stop_screen_share(state: State<'_, AppState>) -> Result<(), String> {
    tracing::info!("Stopping screen share");
    *state.is_screen_sharing.lock().await = false;
    Ok(())
}
