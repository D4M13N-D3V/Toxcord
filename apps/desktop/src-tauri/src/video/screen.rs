//! Screen capture for screen sharing via ToxAV.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use tracing::{error, info, warn};
use xcap::Monitor;

use super::capture::{VideoCaptureError, VideoFrameData};
use super::convert::rgba_to_yuv420;
use super::{VideoError, VideoResult, DEFAULT_VIDEO_FPS};

/// Screen information for selection UI.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScreenInfo {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

/// Screen capture for sharing screen content.
/// Captures screen frames and converts to YUV420 for ToxAV.
pub struct ScreenCapture {
    _thread: thread::JoinHandle<()>,
    running: Arc<AtomicBool>,
}

impl ScreenCapture {
    /// List available screens/monitors.
    pub fn list_screens() -> VideoResult<Vec<ScreenInfo>> {
        info!("SCREEN: Listing available monitors");

        let monitors = Monitor::all()
            .map_err(|e| VideoError::Init(format!("Failed to enumerate monitors: {e}")))?;

        let screens: Vec<ScreenInfo> = monitors
            .iter()
            .enumerate()
            .map(|(idx, monitor)| {
                let name = monitor.name().to_string();
                let is_primary = monitor.is_primary();

                info!(
                    "SCREEN: Found monitor {}: {} ({}x{}) primary={}",
                    idx,
                    name,
                    monitor.width(),
                    monitor.height(),
                    is_primary
                );

                ScreenInfo {
                    id: idx as u32,
                    name,
                    width: monitor.width(),
                    height: monitor.height(),
                    is_primary,
                }
            })
            .collect();

        Ok(screens)
    }

    /// Start capturing a specific screen (or primary if None).
    pub fn start(
        screen_id: Option<u32>,
        frame_tx: mpsc::UnboundedSender<VideoFrameData>,
        error_tx: mpsc::UnboundedSender<VideoCaptureError>,
    ) -> VideoResult<Self> {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let thread = thread::Builder::new()
            .name("screen-capture".into())
            .spawn(move || {
                if let Err(e) = Self::capture_loop(screen_id, frame_tx, running_clone) {
                    error!("Screen capture error: {e}");
                    let _ = error_tx.send(VideoCaptureError {
                        message: e.to_string(),
                    });
                }
            })
            .map_err(|e| VideoError::Init(format!("Failed to spawn screen capture thread: {e}")))?;

        info!("Screen capture started");
        Ok(Self {
            _thread: thread,
            running,
        })
    }

    fn capture_loop(
        screen_id: Option<u32>,
        frame_tx: mpsc::UnboundedSender<VideoFrameData>,
        running: Arc<AtomicBool>,
    ) -> VideoResult<()> {
        info!("SCREEN: Starting capture loop for screen {:?}", screen_id);

        // Get the list of monitors
        let monitors = Monitor::all()
            .map_err(|e| VideoError::Init(format!("Failed to enumerate monitors: {e}")))?;

        if monitors.is_empty() {
            return Err(VideoError::Init("No monitors found".to_string()));
        }

        // Select the monitor
        let monitor = if let Some(id) = screen_id {
            monitors
                .into_iter()
                .nth(id as usize)
                .ok_or_else(|| VideoError::Init(format!("Monitor {} not found", id)))?
        } else {
            // Find primary or use first
            monitors
                .into_iter()
                .find(|m| m.is_primary())
                .or_else(|| Monitor::all().ok()?.into_iter().next())
                .ok_or_else(|| VideoError::Init("No monitor available".to_string()))?
        };

        let monitor_name = monitor.name().to_string();
        info!(
            "SCREEN: Capturing from '{}' ({}x{})",
            monitor_name,
            monitor.width(),
            monitor.height()
        );

        let frame_interval = Duration::from_millis(1000 / DEFAULT_VIDEO_FPS as u64);
        let mut last_frame_time = Instant::now();
        let mut frame_count = 0u64;

        while running.load(Ordering::Relaxed) {
            // Rate limiting
            let elapsed = last_frame_time.elapsed();
            if elapsed < frame_interval {
                thread::sleep(frame_interval - elapsed);
            }
            last_frame_time = Instant::now();

            // Capture screen
            let image = match monitor.capture_image() {
                Ok(img) => img,
                Err(e) => {
                    warn!("SCREEN: Failed to capture frame: {e}");
                    continue;
                }
            };

            let width = image.width() as usize;
            let height = image.height() as usize;

            // xcap returns RGBA data
            let rgba_data = image.as_raw();

            // Convert RGBA to YUV420
            let (y, u, v) = rgba_to_yuv420(rgba_data, width, height);

            let frame_data = VideoFrameData {
                y,
                u,
                v,
                width: width as u16,
                height: height as u16,
            };

            // Send frame
            if frame_tx.send(frame_data).is_err() {
                info!("SCREEN: Receiver dropped, stopping capture");
                break;
            }

            frame_count += 1;
            if frame_count <= 3 {
                info!("SCREEN: Sent frame {} ({}x{})", frame_count, width, height);
            }
        }

        info!("Screen capture loop ended");
        Ok(())
    }

    /// Check if capture is still running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Stop capturing.
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
        info!("Screen capture stopped");
    }
}

impl Drop for ScreenCapture {
    fn drop(&mut self) {
        self.stop();
    }
}
