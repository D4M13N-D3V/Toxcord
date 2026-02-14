//! Audio capture from microphone using cpal.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, SampleFormat, SampleRate, Stream, StreamConfig};
use rubato::{FftFixedInOut, Resampler};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use super::{AudioDevice, AudioError, AudioResult, TOXAV_CHANNELS, TOXAV_SAMPLE_RATE, TOXAV_SAMPLES_PER_FRAME};

/// Audio capture from microphone.
/// Captures audio and resamples to ToxAV format (48kHz mono).
pub struct AudioCapture {
    _stream: Stream,
    running: Arc<AtomicBool>,
}

impl AudioCapture {
    /// Start capturing audio from the default input device.
    ///
    /// Returns a receiver that will receive audio frames as `Vec<i16>`.
    /// Each frame contains TOXAV_SAMPLES_PER_FRAME samples at 48kHz mono.
    pub fn start(
        frame_tx: mpsc::UnboundedSender<Vec<i16>>,
    ) -> AudioResult<Self> {
        Self::start_with_device(None, frame_tx)
    }

    /// Start capturing audio from a specific device (or default if None).
    pub fn start_with_device(
        device_id: Option<&str>,
        frame_tx: mpsc::UnboundedSender<Vec<i16>>,
    ) -> AudioResult<Self> {
        let host = cpal::default_host();

        let device = match device_id {
            Some(id) => Self::find_device(&host, id)?,
            None => host
                .default_input_device()
                .ok_or_else(|| AudioError::DeviceNotFound("No default input device".into()))?,
        };

        let device_name = device.name().unwrap_or_else(|_| "Unknown".into());
        info!("Starting audio capture on: {}", device_name);

        let supported_config = device
            .default_input_config()
            .map_err(|e| AudioError::Init(format!("Failed to get input config: {e}")))?;

        let sample_format = supported_config.sample_format();
        let config: StreamConfig = supported_config.into();
        let input_sample_rate = config.sample_rate.0;
        let input_channels = config.channels as usize;

        debug!(
            "Input config: {} Hz, {} channels, {:?}",
            input_sample_rate, input_channels, sample_format
        );

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        // Create resampler if needed
        let needs_resample = input_sample_rate != TOXAV_SAMPLE_RATE;
        let mut resampler = if needs_resample {
            Some(Self::create_resampler(input_sample_rate, input_channels)?)
        } else {
            None
        };

        // Buffer for accumulating samples
        let mut sample_buffer: Vec<f32> = Vec::new();
        let target_samples = TOXAV_SAMPLES_PER_FRAME * input_channels;

        let stream = match sample_format {
            SampleFormat::F32 => Self::build_stream::<f32>(
                &device,
                &config,
                frame_tx,
                running_clone,
                input_channels,
                &mut resampler,
                &mut sample_buffer,
                target_samples,
            )?,
            SampleFormat::I16 => Self::build_stream::<i16>(
                &device,
                &config,
                frame_tx,
                running_clone,
                input_channels,
                &mut resampler,
                &mut sample_buffer,
                target_samples,
            )?,
            SampleFormat::U16 => Self::build_stream::<u16>(
                &device,
                &config,
                frame_tx,
                running_clone,
                input_channels,
                &mut resampler,
                &mut sample_buffer,
                target_samples,
            )?,
            _ => {
                return Err(AudioError::Init(format!(
                    "Unsupported sample format: {:?}",
                    sample_format
                )));
            }
        };

        stream
            .play()
            .map_err(|e| AudioError::Stream(format!("Failed to start stream: {e}")))?;

        info!("Audio capture started");
        Ok(Self {
            _stream: stream,
            running,
        })
    }

    fn find_device(host: &Host, device_id: &str) -> AudioResult<Device> {
        let devices = host
            .input_devices()
            .map_err(|e| AudioError::Init(format!("Failed to enumerate devices: {e}")))?;

        for device in devices {
            if let Ok(name) = device.name() {
                if name == device_id {
                    return Ok(device);
                }
            }
        }

        Err(AudioError::DeviceNotFound(device_id.to_string()))
    }

    fn create_resampler(
        input_rate: u32,
        _channels: usize,
    ) -> AudioResult<FftFixedInOut<f32>> {
        // Create resampler from input rate to 48kHz
        let resampler = FftFixedInOut::<f32>::new(
            input_rate as usize,
            TOXAV_SAMPLE_RATE as usize,
            1024, // chunk size
            1,    // mono output
        )
        .map_err(|e| AudioError::Resample(format!("Failed to create resampler: {e}")))?;

        debug!(
            "Created resampler: {} Hz -> {} Hz",
            input_rate, TOXAV_SAMPLE_RATE
        );
        Ok(resampler)
    }

    fn build_stream<T: cpal::Sample + cpal::SizedSample + Send + 'static>(
        device: &Device,
        config: &StreamConfig,
        frame_tx: mpsc::UnboundedSender<Vec<i16>>,
        running: Arc<AtomicBool>,
        input_channels: usize,
        resampler: &mut Option<FftFixedInOut<f32>>,
        sample_buffer: &mut Vec<f32>,
        target_samples: usize,
    ) -> AudioResult<Stream>
    where
        f32: cpal::FromSample<T>,
    {
        // We need to move these into the closure
        let mut buffer = std::mem::take(sample_buffer);
        let mut resamp = resampler.take();

        let stream = device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    if !running.load(Ordering::Relaxed) {
                        return;
                    }

                    // Convert samples to f32
                    for sample in data {
                        let f: f32 = cpal::Sample::from_sample(*sample);
                        buffer.push(f);
                    }

                    // Process complete frames
                    while buffer.len() >= target_samples {
                        let frame_data: Vec<f32> = buffer.drain(..target_samples).collect();

                        // Mix down to mono if stereo
                        let mono: Vec<f32> = if input_channels > 1 {
                            frame_data
                                .chunks(input_channels)
                                .map(|chunk| chunk.iter().sum::<f32>() / input_channels as f32)
                                .collect()
                        } else {
                            frame_data
                        };

                        // Resample if needed
                        let resampled = if let Some(ref mut r) = resamp {
                            match r.process(&[mono], None) {
                                Ok(output) => output.into_iter().next().unwrap_or_default(),
                                Err(e) => {
                                    warn!("Resample error: {e}");
                                    continue;
                                }
                            }
                        } else {
                            mono
                        };

                        // Convert to i16 for ToxAV
                        let pcm: Vec<i16> = resampled
                            .iter()
                            .map(|&s| (s * 32767.0).clamp(-32768.0, 32767.0) as i16)
                            .collect();

                        // Send frame
                        if frame_tx.send(pcm).is_err() {
                            // Receiver dropped, stop capturing
                            return;
                        }
                    }
                },
                move |err| {
                    error!("Audio capture error: {err}");
                },
                None,
            )
            .map_err(|e| AudioError::StreamBuild(format!("Failed to build stream: {e}")))?;

        Ok(stream)
    }

    /// List available input devices
    pub fn list_devices() -> AudioResult<Vec<AudioDevice>> {
        let host = cpal::default_host();
        let default_device_name = host
            .default_input_device()
            .and_then(|d| d.name().ok());

        let devices = host
            .input_devices()
            .map_err(|e| AudioError::Init(format!("Failed to enumerate devices: {e}")))?;

        let mut result = Vec::new();
        for device in devices {
            if let Ok(name) = device.name() {
                let is_default = default_device_name.as_ref() == Some(&name);
                result.push(AudioDevice {
                    id: name.clone(),
                    name,
                    is_default,
                });
            }
        }

        Ok(result)
    }

    /// Check if capture is still running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Stop capturing
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
        info!("Audio capture stopped");
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        self.stop();
    }
}
