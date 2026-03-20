use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// A single playing voice — wraps a rodio Sink.
struct Voice {
    sink: Sink,
}

/// Manages polyphonic sine tone synthesis with a hard voice limit.
pub struct VoicePool {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    voices: Arc<Mutex<Vec<Voice>>>,
    max_voices: usize,
}

impl VoicePool {
    /// Create a new voice pool connected to the default audio device.
    pub fn new(max_voices: usize) -> Option<Self> {
        let (stream, stream_handle) = OutputStream::try_default().ok()?;
        Some(Self {
            _stream: stream,
            stream_handle,
            voices: Arc::new(Mutex::new(Vec::new())),
            max_voices,
        })
    }

    /// Trigger a sine tone at `frequency` Hz.
    ///
    /// Volume is clamped to [0, 1].  Duration is fixed at ~150 ms.
    /// If the voice limit is reached, the oldest voice is stopped.
    pub fn play_tone(&self, frequency: f32, volume: f32) {
        let mut voices = self.voices.lock().unwrap();

        // Cull finished voices
        voices.retain(|v| !v.sink.empty());

        // Enforce voice limit — drop oldest
        if voices.len() >= self.max_voices {
            voices.remove(0);
        }

        let source = SineSource::new(frequency, volume.clamp(0.0, 1.0), Duration::from_millis(150));

        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
            sink.append(source);
            voices.push(Voice { sink });
        }
    }
}

// ---------------------------------------------------------------------------
// Manual sine wave AudioSource implementation (no fundsp dependency needed)
// ---------------------------------------------------------------------------

use rodio::Source;

/// A short sine tone with exponential decay envelope (~150 ms).
struct SineSource {
    frequency: f32,
    amplitude: f32,
    sample_rate: u32,
    total_samples: u32,
    current_sample: u32,
}

impl SineSource {
    fn new(frequency: f32, amplitude: f32, duration: Duration) -> Self {
        let sample_rate = 44100u32;
        let total_samples = (sample_rate as f32 * duration.as_secs_f32()) as u32;
        Self {
            frequency,
            amplitude,
            sample_rate,
            total_samples,
            current_sample: 0,
        }
    }
}

impl Iterator for SineSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.current_sample >= self.total_samples {
            return None;
        }

        let t = self.current_sample as f32 / self.sample_rate as f32;

        // Exponential decay envelope
        let decay_rate = 20.0_f32; // higher = faster decay
        let envelope = (1.0 - self.current_sample as f32 / self.total_samples as f32)
            * (-decay_rate * t).exp();

        let sample = (2.0 * std::f32::consts::PI * self.frequency * t).sin()
            * self.amplitude
            * envelope;

        self.current_sample += 1;
        Some(sample)
    }
}

impl Source for SineSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(
            self.total_samples as f32 / self.sample_rate as f32,
        ))
    }
}
