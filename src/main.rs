use bevy::prelude::*;

mod audio;
mod input;
mod physics;
mod rendering;

/// Entry point — assembles the Bevy app with all plugins.
fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Resonance — Particle Physics Sonification".into(),
                        resolution: (1280.0, 720.0).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(physics::PhysicsPlugin)
        .add_plugins(audio::AudioPlugin)
        .add_plugins(rendering::RenderingPlugin)
        .add_plugins(input::InputPlugin)
        .run();
}
