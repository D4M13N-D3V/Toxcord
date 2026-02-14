//! Audio playback to speakers using cpal.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, SampleFormat, Stream, StreamConfig};
use tracing::{debug, error, info, trace};

use super::mixer::AudioMixer;
use super::{AudioDevice, AudioError, AudioResult, TOXAV_SAMPLE_RATE};

/// Audio playback to speakers.
/// Plays audio from the mixer which combines multiple sources.
pub struct AudioPlayback {
    _stream: Stream,
    running: Arc<AtomicBool>,
}

impl AudioPlayback {
    /// Start audio playback on the default output device.
    ///
    /// Takes a shared mixer that combines audio from multiple sources.
    pub fn start(mixer: Arc<Mutex<AudioMixer>>) -> AudioResult<Self> {
        Self::start_with_device(None, mixer)
    }

    /// Start audio playback on a specific device (or default if None).
    pub fn start_with_device(
        device_id: Option<&str>,
        mixer: Arc<Mutex<AudioMixer>>,
    ) -> AudioResult<Self> {
        info!("AudioPlayback::start_with_device called");
        let host = cpal::default_host();
        info!("Got cpal host: {:?}", host.id());

        let device = match device_id {
            Some(id) => Self::find_device(&host, id)?,
            None => host
                .default_output_device()
                .ok_or_else(|| AudioError::DeviceNotFound("No default output device".into()))?,
        };

        let device_name = device.name().unwrap_or_else(|_| "Unknown".into());
        info!("Starting audio playback on: {}", device_name);

        // Try to use 48kHz if supported, otherwise use default
        let supported_configs = device
            .supported_output_configs()
            .map_err(|e| AudioError::Init(format!("Failed to get output configs: {e}")))?;

        let mut selected_config = None;
        for config_range in supported_configs {
            if config_range.min_sample_rate().0 <= TOXAV_SAMPLE_RATE
                && config_range.max_sample_rate().0 >= TOXAV_SAMPLE_RATE
            {
                selected_config = Some(config_range.with_sample_rate(cpal::SampleRate(TOXAV_SAMPLE_RATE)));
                break;
            }
        }

        // Use selected config or fall back to default
        let supported_config = match selected_config {
            Some(config) => config,
            None => device
                .default_output_config()
                .map_err(|e| AudioError::Init(format!("Failed to get output config: {e}")))?,
        };

        let sample_format = supported_config.sample_format();
        let config: StreamConfig = supported_config.into();
        let output_sample_rate = config.sample_rate.0;
        let output_channels = config.channels as usize;

        debug!(
            "Output config: {} Hz, {} channels, {:?}",
            output_sample_rate, output_channels, sample_format
        );

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        info!("Building output stream with format: {:?}", sample_format);
        let stream = match sample_format {
            SampleFormat::F32 => Self::build_stream::<f32>(
                &device,
                &config,
                mixer,
                running_clone,
                output_channels,
            )?,
            SampleFormat::I16 => Self::build_stream::<i16>(
                &device,
                &config,
                mixer,
                running_clone,
                output_channels,
            )?,
            SampleFormat::U16 => Self::build_stream::<u16>(
                &device,
                &config,
                mixer,
                running_clone,
                output_channels,
            )?,
            SampleFormat::I8 => Self::build_stream::<i8>(
                &device,
                &config,
                mixer,
                running_clone,
                output_channels,
            )?,
            SampleFormat::U8 => Self::build_stream::<u8>(
                &device,
                &config,
                mixer,
                running_clone,
                output_channels,
            )?,
            SampleFormat::I32 => Self::build_stream::<i32>(
                &device,
                &config,
                mixer,
                running_clone,
                output_channels,
            )?,
            SampleFormat::U32 => Self::build_stream::<u32>(
                &device,
                &config,
                mixer,
                running_clone,
                output_channels,
            )?,
            SampleFormat::F64 => Self::build_stream::<f64>(
                &device,
                &config,
                mixer,
                running_clone,
                output_channels,
            )?,
            _ => {
                return Err(AudioError::Init(format!(
                    "Unsupported sample format: {:?}",
                    sample_format
                )));
            }
        };
        info!("Output stream built successfully");

        stream
            .play()
            .map_err(|e| AudioError::Stream(format!("Failed to start stream: {e}")))?;

        info!("Audio playback started on device: {}, sample_rate: {}, channels: {}",
              device_name, output_sample_rate, output_channels);
        Ok(Self {
            _stream: stream,
            running,
        })
    }

    fn find_device(host: &Host, device_id: &str) -> AudioResult<Device> {
        let devices = host
            .output_devices()
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

    fn build_stream<T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32> + Send + 'static>(
        device: &Device,
        config: &StreamConfig,
        mixer: Arc<Mutex<AudioMixer>>,
        running: Arc<AtomicBool>,
        output_channels: usize,
    ) -> AudioResult<Stream> {
        use std::sync::atomic::AtomicUsize;
        static CALLBACK_COUNT: AtomicUsize = AtomicUsize::new(0);

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    if !running.load(Ordering::Relaxed) {
                        // Fill with silence
                        for sample in data.iter_mut() {
                            *sample = T::EQUILIBRIUM;
                        }
                        return;
                    }

                    // Log every 50 callbacks (~1 second at typical rates)
                    let count = CALLBACK_COUNT.fetch_add(1, Ordering::Relaxed);
                    if count % 50 == 0 {
                        debug!("Playback callback #{}: requesting {} samples, channels={}",
                               count, data.len() / output_channels, output_channels);
                    }

                    // Calculate how many mono samples we need
                    let samples_needed = data.len() / output_channels;

                    // Try to get mixed audio (non-blocking)
                    let (mixed, has_audio, lock_acquired) = {
                        // Use try_lock to avoid blocking the audio thread
                        if let Ok(mut m) = mixer.try_lock() {
                            let output = m.get_mixed_output(samples_needed);
                            // Check if there's any non-zero audio
                            let has_audio = output.iter().any(|&s| s != 0);
                            (output, has_audio, true)
                        } else {
                            // Couldn't get lock, output silence
                            (vec![0i16; samples_needed], false, false)
                        }
                    };

                    // Log lock failures and audio presence periodically
                    if count % 50 == 0 {
                        debug!("Playback: lock_acquired={}, has_audio={}", lock_acquired, has_audio);
                    }

                    if has_audio {
                        trace!("Playback: outputting {} samples with audio", samples_needed);
                    }

                    // Convert and write to output
                    let mut sample_idx = 0;
                    for chunk in data.chunks_mut(output_channels) {
                        let sample = if sample_idx < mixed.len() {
                            // Convert i16 to f32 in range [-1, 1]
                            mixed[sample_idx] as f32 / 32768.0
                        } else {
                            0.0
                        };

                        // Write same sample to all output channels (mono -> stereo expansion)
                        for output in chunk.iter_mut() {
                            *output = T::from_sample(sample);
                        }
                        sample_idx += 1;
                    }
                },
                move |err| {
                    error!("Audio playback error: {err}");
                },
                None,
            )
            .map_err(|e| AudioError::StreamBuild(format!("Failed to build stream: {e}")))?;

        Ok(stream)
    }

    /// List available output devices
    pub fn list_devices() -> AudioResult<Vec<AudioDevice>> {
        let host = cpal::default_host();
        let default_device_name = host
            .default_output_device()
            .and_then(|d| d.name().ok());

        let devices = host
            .output_devices()
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

    /// Check if playback is still running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Stop playback
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
        info!("Audio playback stopped");
    }
}

impl Drop for AudioPlayback {
    fn drop(&mut self) {
        self.stop();
    }
}
