use std::sync::mpsc::{self, SyncSender};
use std::thread;
use std::time::Duration;

use rodio::{OutputStream, Sink, Source};

/// Request sent to the audio thread.
struct ToneRequest {
    frequency: f32,
    amplitude: f32,
}

/// Thread-safe handle to the background audio thread.
///
/// `rodio::OutputStream` is `!Send`, so it must stay on the thread that created it.
/// This struct holds only a `SyncSender` which is `Send + Sync`, making `VoicePool`
/// usable as a Bevy `Resource`.
pub struct VoicePool {
    sender: SyncSender<ToneRequest>,
}

impl VoicePool {
    /// Spawn the audio background thread and return a handle.
    ///
    /// Returns `None` if the default audio device cannot be opened.
    pub fn new(max_voices: usize) -> Option<Self> {
        // Probe device availability before spawning the thread.
        OutputStream::try_default().ok()?;

        // Bounded channel — full channel drops tones rather than blocking the game.
        let (sender, receiver) = mpsc::sync_channel::<ToneRequest>(64);

        thread::spawn(move || {
            // `_stream` must be held alive for the duration of the thread.
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(pair) => pair,
                Err(_) => return,
            };

            let mut sinks: Vec<Sink> = Vec::new();

            while let Ok(req) = receiver.recv() {
                // Cull finished sinks
                sinks.retain(|s| !s.empty());

                // Enforce voice limit — drop oldest
                if sinks.len() >= max_voices {
                    sinks.remove(0);
                }

                if let Ok(sink) = Sink::try_new(&stream_handle) {
                    let source = SineSource::new(
                        req.frequency,
                        req.amplitude.clamp(0.0, 1.0),
                        Duration::from_millis(150),
                    );
                    sink.append(source);
                    sinks.push(sink);
                }
            }
            // Receiver dropped — thread exits cleanly.
        });

        Some(Self { sender })
    }

    /// Queue a sine tone at `frequency` Hz with the given `volume` (0–1).
    ///
    /// Non-blocking — silently drops the tone if the channel is full.
    pub fn play_tone(&self, frequency: f32, volume: f32) {
        let _ = self.sender.try_send(ToneRequest {
            frequency,
            amplitude: volume,
        });
    }
}

// ---------------------------------------------------------------------------
// Manual sine-wave Source implementation
// ---------------------------------------------------------------------------

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

        // Exponential decay envelope: fast attack, tail fade
        let decay = (-20.0_f32 * t).exp();
        let sample =
            (2.0 * std::f32::consts::PI * self.frequency * t).sin() * self.amplitude * decay;

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
