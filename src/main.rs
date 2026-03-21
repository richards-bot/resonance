use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

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
                }),
        )
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(physics::PhysicsPlugin)
        .add_plugins(audio::AudioPlugin)
        .add_plugins(rendering::RenderingPlugin)
        .add_plugins(input::InputPlugin)
        .add_systems(Startup, setup_lighting)
        .run();
}

/// Spawn ambient and directional lights for PBR rendering.
fn setup_lighting(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 400.0,
    });
    commands.spawn(DirectionalLight {
        illuminance: 20_000.0,
        shadows_enabled: false,
        ..default()
    });
    commands.spawn((
        DirectionalLight { illuminance: 8_000.0, shadows_enabled: false, ..default() },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -2.0, 1.0, 0.0)),
    ));
}
