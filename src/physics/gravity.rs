use bevy::prelude::*;

use super::particles::{GravityStrength, Particle};

/// A gravity well that attracts nearby particles.
#[derive(Component)]
pub struct GravityWell {
    /// Strength multiplier relative to global setting.
    pub strength: f32,
}

/// Gravitational constant used in F = G * mass / r^2 calculations.
const G: f32 = 1.0;

/// Minimum distance to avoid singularity in gravity calculation.
const MIN_DIST: f32 = 20.0;

/// Apply gravitational attraction from every well to every particle.
///
/// Uses F = G * global_strength * well_strength * mass / r² clamped at MIN_DIST.
pub fn apply_gravity(
    wells: Query<(&Transform, &GravityWell)>,
    mut particles: Query<(&mut Particle, &Transform)>,
    strength: Res<GravityStrength>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut particle, p_transform) in &mut particles {
        let p_pos = p_transform.translation.truncate();

        for (w_transform, well) in &wells {
            let w_pos = w_transform.translation.truncate();
            let delta = w_pos - p_pos;
            let dist = delta.length().max(MIN_DIST);
            let force_mag = G * strength.0 * well.strength * particle.mass / (dist * dist);
            let accel = delta.normalize() * force_mag / particle.mass;
            particle.velocity += accel * dt;
        }
    }
}
