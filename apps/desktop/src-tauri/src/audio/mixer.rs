//! Audio mixer for combining multiple audio sources.
//!
//! Used for voice channels where audio from multiple participants
//! needs to be mixed together for playback.

use std::collections::HashMap;
use std::collections::VecDeque;

use tracing::debug;

use super::TOXAV_SAMPLES_PER_FRAME;

/// Maximum number of samples to buffer per source (to handle jitter)
const MAX_BUFFER_SAMPLES: usize = TOXAV_SAMPLES_PER_FRAME * 10; // ~200ms buffer

/// Audio source representing one peer's audio stream
struct AudioSource {
    /// Ring buffer of PCM samples
    buffer: VecDeque<i16>,
    /// Running average for audio level calculation
    level_accumulator: f32,
    level_sample_count: usize,
}

impl AudioSource {
    fn new() -> Self {
        Self {
            buffer: VecDeque::with_capacity(MAX_BUFFER_SAMPLES),
            level_accumulator: 0.0,
            level_sample_count: 0,
        }
    }

    fn push_samples(&mut self, samples: &[i16]) {
        // Add new samples
        for &sample in samples {
            // Evict old samples if buffer is full
            if self.buffer.len() >= MAX_BUFFER_SAMPLES {
                self.buffer.pop_front();
            }
            self.buffer.push_back(sample);

            // Update level calculation
            let abs_sample = (sample as f32).abs() / 32768.0;
            self.level_accumulator += abs_sample;
            self.level_sample_count += 1;
        }
    }

    fn get_samples(&mut self, count: usize) -> Vec<i16> {
        let available = self.buffer.len().min(count);
        let mut result = Vec::with_capacity(count);

        // Get available samples
        for _ in 0..available {
            if let Some(sample) = self.buffer.pop_front() {
                result.push(sample);
            }
        }

        // Pad with silence if not enough samples
        result.resize(count, 0);
        result
    }

    fn available_samples(&self) -> usize {
        self.buffer.len()
    }

    /// Get current audio level (0.0 - 1.0)
    fn get_level(&mut self) -> f32 {
        if self.level_sample_count == 0 {
            return 0.0;
        }

        let level = self.level_accumulator / self.level_sample_count as f32;
        // Reset for next period
        self.level_accumulator = 0.0;
        self.level_sample_count = 0;

        // Apply some smoothing/scaling
        (level * 3.0).min(1.0)
    }
}

/// Audio mixer that combines multiple audio sources into one output stream.
pub struct AudioMixer {
    /// Audio sources keyed by friend_number
    sources: HashMap<u32, AudioSource>,
    /// Output sample rate
    sample_rate: u32,
    /// Whether mixer is muted (deafened)
    muted: bool,
}

impl AudioMixer {
    /// Create a new audio mixer
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sources: HashMap::new(),
            sample_rate,
            muted: false,
        }
    }

    /// Push an audio frame from a source
    pub fn push_frame(&mut self, friend_number: u32, pcm: Vec<i16>) {
        let source = self.sources.entry(friend_number).or_insert_with(AudioSource::new);
        source.push_samples(&pcm);
    }

    /// Get mixed audio for playback
    ///
    /// Returns `sample_count` samples of mixed audio from all sources.
    pub fn get_mixed_output(&mut self, sample_count: usize) -> Vec<i16> {
        if self.muted {
            return vec![0i16; sample_count];
        }

        if self.sources.is_empty() {
            return vec![0i16; sample_count];
        }

        // Collect samples from all sources
        let source_count = self.sources.len();
        let mut all_samples: Vec<Vec<i16>> = Vec::with_capacity(source_count);

        for source in self.sources.values_mut() {
            all_samples.push(source.get_samples(sample_count));
        }

        // Mix all sources together
        let mut mixed = vec![0i32; sample_count];
        for source_samples in &all_samples {
            for (i, &sample) in source_samples.iter().enumerate() {
                mixed[i] += sample as i32;
            }
        }

        // Normalize and clamp to i16 range
        // Simple averaging to prevent clipping
        let divisor = source_count.max(1) as i32;
        mixed
            .into_iter()
            .map(|s| (s / divisor).clamp(-32768, 32767) as i16)
            .collect()
    }

    /// Remove a source
    pub fn remove_source(&mut self, friend_number: u32) {
        self.sources.remove(&friend_number);
        debug!("Removed audio source for friend {}", friend_number);
    }

    /// Get audio level for a specific source (0.0 - 1.0)
    pub fn get_level(&mut self, friend_number: u32) -> f32 {
        self.sources
            .get_mut(&friend_number)
            .map(|s| s.get_level())
            .unwrap_or(0.0)
    }

    /// Get all source levels
    pub fn get_all_levels(&mut self) -> HashMap<u32, f32> {
        self.sources
            .iter_mut()
            .map(|(&k, v)| (k, v.get_level()))
            .collect()
    }

    /// Check if a source has audio buffered
    pub fn has_audio(&self, friend_number: u32) -> bool {
        self.sources
            .get(&friend_number)
            .map(|s| s.available_samples() > 0)
            .unwrap_or(false)
    }

    /// Set muted state (deafen)
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    /// Check if muted
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Clear all sources
    pub fn clear(&mut self) {
        self.sources.clear();
    }

    /// Get number of active sources
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new(super::TOXAV_SAMPLE_RATE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixer_empty() {
        let mut mixer = AudioMixer::new(48000);
        let output = mixer.get_mixed_output(960);
        assert_eq!(output.len(), 960);
        assert!(output.iter().all(|&s| s == 0));
    }

    #[test]
    fn test_mixer_single_source() {
        let mut mixer = AudioMixer::new(48000);
        let samples: Vec<i16> = (0..960).map(|i| (i % 100) as i16).collect();
        mixer.push_frame(1, samples.clone());

        let output = mixer.get_mixed_output(960);
        assert_eq!(output.len(), 960);
        assert_eq!(output, samples);
    }

    #[test]
    fn test_mixer_multiple_sources() {
        let mut mixer = AudioMixer::new(48000);

        // Two sources with simple values
        mixer.push_frame(1, vec![100i16; 960]);
        mixer.push_frame(2, vec![100i16; 960]);

        let output = mixer.get_mixed_output(960);
        assert_eq!(output.len(), 960);
        // Average of two 100s is 100
        assert!(output.iter().all(|&s| s == 100));
    }

    #[test]
    fn test_mixer_muted() {
        let mut mixer = AudioMixer::new(48000);
        mixer.push_frame(1, vec![1000i16; 960]);
        mixer.set_muted(true);

        let output = mixer.get_mixed_output(960);
        assert!(output.iter().all(|&s| s == 0));
    }
}
