use bevy::math::Isometry3d;
use bevy::prelude::*;

pub mod particles;
pub mod star;
pub mod ui;

use crate::input::MoonTarget;
use crate::physics::bodies::Planet;

/// Bevy plugin that sets up all rendering systems.
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.08)))
            .add_systems(Startup, ui::setup_hud)
            .add_systems(
                Update,
                (
                    particles::draw_trails,
                    particles::draw_body_trails,
                    star::animate_star,
                    ui::update_hud,
                    highlight_target,
                ),
            );
    }
}

/// Draw a pulsing yellow ring around the planet currently targeted for moon placement.
///
/// Ring radius = planet visual radius + 20, with a ±5 unit sinusoidal pulse.
/// Clears `MoonTarget` if the targeted entity has been despawned.
fn highlight_target(
    mut moon_target: ResMut<MoonTarget>,
    planets: Query<(&Transform, &Planet)>,
    mut gizmos: Gizmos,
    time: Res<Time>,
) {
    let Some(target_entity) = moon_target.planet else { return };
    let Ok((transform, planet)) = planets.get(target_entity) else {
        moon_target.planet = None;
        return;
    };

    // Invert the spawn formula: mass = r^3 * 0.01  →  r = cbrt(mass / 0.01)
    let planet_radius = (planet.mass / 0.01_f32).cbrt();
    let pulse = (time.elapsed_secs() * 3.0).sin() * 5.0;
    let ring_radius = planet_radius + 20.0 + pulse;

    gizmos.circle(
        Isometry3d::new(transform.translation, Quat::IDENTITY),
        ring_radius,
        Color::srgba(1.0, 1.0, 0.2, 0.8),
    );
}
