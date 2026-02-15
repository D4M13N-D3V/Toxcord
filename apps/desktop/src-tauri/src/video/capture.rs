//! Video capture from camera using nokhwa.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution};
use nokhwa::Camera;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use super::convert::rgb_to_yuv420;
use super::{VideoDevice, VideoError, VideoResult, DEFAULT_VIDEO_FPS, DEFAULT_VIDEO_HEIGHT, DEFAULT_VIDEO_WIDTH};

/// Video frame data in YUV420 format ready for ToxAV.
#[derive(Debug, Clone)]
pub struct VideoFrameData {
    pub y: Vec<u8>,
    pub u: Vec<u8>,
    pub v: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

/// Video capture error sent via channel
#[derive(Debug, Clone)]
pub struct VideoCaptureError {
    pub message: String,
}

/// Video capture from camera.
/// Captures video frames and converts to YUV420 for ToxAV.
pub struct VideoCapture {
    _thread: thread::JoinHandle<()>,
    running: Arc<AtomicBool>,
}

impl VideoCapture {
    /// Start capturing video from the default camera.
    pub fn start(
        frame_tx: mpsc::UnboundedSender<VideoFrameData>,
        error_tx: mpsc::UnboundedSender<VideoCaptureError>,
    ) -> VideoResult<Self> {
        Self::start_with_device(None, frame_tx, error_tx)
    }

    /// Start capturing video from a specific device (or default if None).
    pub fn start_with_device(
        device_index: Option<u32>,
        frame_tx: mpsc::UnboundedSender<VideoFrameData>,
        error_tx: mpsc::UnboundedSender<VideoCaptureError>,
    ) -> VideoResult<Self> {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let index = device_index.unwrap_or(0);

        let thread = thread::Builder::new()
            .name("video-capture".into())
            .spawn(move || {
                if let Err(e) = Self::capture_loop(index, frame_tx, running_clone) {
                    error!("Video capture error: {e}");
                    // Send error to main thread so it can emit to frontend
                    let _ = error_tx.send(VideoCaptureError {
                        message: e.to_string(),
                    });
                }
            })
            .map_err(|e| VideoError::Init(format!("Failed to spawn capture thread: {e}")))?;

        info!("Video capture started");
        Ok(Self {
            _thread: thread,
            running,
        })
    }

    fn capture_loop(
        device_index: u32,
        frame_tx: mpsc::UnboundedSender<VideoFrameData>,
        running: Arc<AtomicBool>,
    ) -> VideoResult<()> {
        info!("CAMERA: Starting capture loop for device index {}", device_index);

        let camera_index = CameraIndex::Index(device_index);

        // Request RGB format at our target resolution
        let target_format = CameraFormat::new(
            Resolution::new(DEFAULT_VIDEO_WIDTH, DEFAULT_VIDEO_HEIGHT),
            FrameFormat::MJPEG, // Most cameras support MJPEG
            DEFAULT_VIDEO_FPS,
        );
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(target_format));

        info!("CAMERA: Opening camera...");
        let mut camera = Camera::new(camera_index, requested)
            .map_err(|e| VideoError::Init(format!("Failed to open camera: {e}")))?;

        info!("CAMERA: Opening stream...");
        camera
            .open_stream()
            .map_err(|e| VideoError::Init(format!("Failed to open camera stream: {e}")))?;

        let resolution = camera.resolution();
        let width = resolution.width() as usize;
        let height = resolution.height() as usize;

        info!(
            "CAMERA: Successfully opened {}x{} @ {} fps",
            width, height, DEFAULT_VIDEO_FPS
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

            // Capture frame
            let frame = match camera.frame() {
                Ok(f) => f,
                Err(e) => {
                    warn!("CAMERA: Failed to capture frame: {e}");
                    continue;
                }
            };

            // Decode to RGB
            let rgb_data = match frame.decode_image::<RgbFormat>() {
                Ok(img) => img.into_raw(),
                Err(e) => {
                    warn!("CAMERA: Failed to decode frame: {e}");
                    continue;
                }
            };

            // Convert to YUV420
            let (y, u, v) = rgb_to_yuv420(&rgb_data, width, height);

            let frame_data = VideoFrameData {
                y,
                u,
                v,
                width: width as u16,
                height: height as u16,
            };

            // Send frame
            if frame_tx.send(frame_data).is_err() {
                // Receiver dropped, stop capturing
                info!("CAMERA: Receiver dropped, stopping capture");
                break;
            }

            frame_count += 1;
            if frame_count <= 3 {
                info!("CAMERA: Sent frame {} ({}x{})", frame_count, width, height);
            }
        }

        info!("Video capture loop ended");
        Ok(())
    }

    /// List available video devices.
    pub fn list_devices() -> VideoResult<Vec<VideoDevice>> {
        // On Linux, try to manually scan /dev/video* first for reliability
        #[cfg(target_os = "linux")]
        {
            info!("CAMERA: Scanning /dev/video* devices on Linux");
            let mut devices = Vec::new();

            // Check for video devices directly
            for i in 0..10 {
                let path = format!("/dev/video{}", i);
                if std::path::Path::new(&path).exists() {
                    // Try to get device name using v4l2 ioctl, or use generic name
                    let name = Self::get_v4l2_device_name(&path).unwrap_or_else(|| format!("Camera {}", i));
                    info!("CAMERA: Found device at {}: {}", path, name);
                    devices.push(VideoDevice {
                        id: i.to_string(),
                        name,
                        is_default: devices.is_empty(),
                    });
                }
            }

            if !devices.is_empty() {
                info!("CAMERA: Found {} devices via /dev scan", devices.len());
                return Ok(devices);
            }

            info!("CAMERA: No /dev/video* devices found, trying nokhwa query");
        }

        // Fallback to nokhwa query
        #[cfg(target_os = "linux")]
        let backend = nokhwa::utils::ApiBackend::Video4Linux;
        #[cfg(not(target_os = "linux"))]
        let backend = nokhwa::utils::ApiBackend::Auto;

        info!("CAMERA: Querying devices with backend: {:?}", backend);

        let devices = match nokhwa::query(backend) {
            Ok(d) => {
                info!("CAMERA: Found {} devices via nokhwa", d.len());
                d
            }
            Err(e) => {
                warn!("CAMERA: Failed to query with {:?} backend: {}", backend, e);
                // Fallback to Auto on Linux if V4L2 fails
                #[cfg(target_os = "linux")]
                {
                    info!("CAMERA: Trying Auto backend as fallback");
                    match nokhwa::query(nokhwa::utils::ApiBackend::Auto) {
                        Ok(d) => d,
                        Err(e2) => {
                            warn!("CAMERA: Auto backend also failed: {}", e2);
                            return Ok(vec![]); // Return empty list instead of error
                        }
                    }
                }
                #[cfg(not(target_os = "linux"))]
                {
                    return Err(VideoError::Init(format!("Failed to query cameras: {e}")));
                }
            }
        };

        for (idx, dev) in devices.iter().enumerate() {
            info!("CAMERA: Device {}: {} (index: {:?})", idx, dev.human_name(), dev.index());
        }

        let result: Vec<VideoDevice> = devices
            .iter()
            .enumerate()
            .map(|(idx, info)| VideoDevice {
                id: idx.to_string(),
                name: info.human_name().to_string(),
                is_default: idx == 0,
            })
            .collect();

        Ok(result)
    }

    /// Check if capture is still running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Stop capturing.
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
        info!("Video capture stopped");
    }
}

impl Drop for VideoCapture {
    fn drop(&mut self) {
        self.stop();
    }
}

impl VideoCapture {
    /// Get the V4L2 device name using ioctl (Linux only)
    #[cfg(target_os = "linux")]
    fn get_v4l2_device_name(path: &str) -> Option<String> {
        use std::fs::File;
        use std::os::unix::io::AsRawFd;

        // VIDIOC_QUERYCAP ioctl number
        const VIDIOC_QUERYCAP: libc::c_ulong = 0x80685600;

        #[repr(C)]
        struct V4l2Capability {
            driver: [u8; 16],
            card: [u8; 32],
            bus_info: [u8; 32],
            version: u32,
            capabilities: u32,
            device_caps: u32,
            reserved: [u32; 3],
        }

        let file = File::open(path).ok()?;
        let fd = file.as_raw_fd();

        let mut cap: V4l2Capability = unsafe { std::mem::zeroed() };

        let result = unsafe { libc::ioctl(fd, VIDIOC_QUERYCAP, &mut cap) };

        if result == 0 {
            // Convert card name to string, stopping at first null byte
            let name_bytes: Vec<u8> = cap.card.iter().take_while(|&&b| b != 0).copied().collect();
            String::from_utf8(name_bytes).ok()
        } else {
            None
        }
    }
}
