use bevy::prelude::*;

pub mod scale;

#[cfg(feature = "audio")]
pub mod synth;

#[cfg(feature = "audio")]
use crate::physics::collision::CollisionEvent;
#[cfg(feature = "audio")]
use synth::VoicePool;

/// Wraps the rodio voice pool as a Bevy resource.
#[cfg(feature = "audio")]
#[derive(Resource)]
pub struct AudioEngine(pub VoicePool);

/// Maximum simultaneous audio voices before oldest is culled.
#[cfg(feature = "audio")]
const MAX_VOICES: usize = 16;

/// Minimum collision speed required to trigger an audio event.
#[cfg(feature = "audio")]
const MIN_SPEED: f32 = 10.0;

/// Speed at which the collision tone reaches maximum volume.
#[cfg(feature = "audio")]
const MAX_SPEED: f32 = 300.0;

/// Bevy plugin for audio synthesis.
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    #[allow(unused_variables)]
    fn build(&self, app: &mut App) {
        #[cfg(feature = "audio")]
        {
            if let Some(pool) = VoicePool::new(MAX_VOICES) {
                app.insert_resource(AudioEngine(pool))
                    .add_systems(Update, handle_collision_audio);
            } else {
                warn!("Failed to open audio device — audio disabled");
            }
        }
    }
}

/// React to collision events by playing sine tones.
#[cfg(feature = "audio")]
fn handle_collision_audio(
    mut events: EventReader<CollisionEvent>,
    engine: Option<ResMut<AudioEngine>>,
) {
    let Some(engine) = engine else { return };

    for event in events.read() {
        if event.speed < MIN_SPEED {
            continue;
        }
        let volume = (event.speed / MAX_SPEED).clamp(0.05, 1.0);
        engine.0.play_tone(event.freq_a, volume);
        if (event.freq_b - event.freq_a).abs() > 1.0 {
            engine.0.play_tone(event.freq_b, volume * 0.7);
        }
    }
}
