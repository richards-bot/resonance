use bevy::prelude::*;

use super::particles::Particle;

/// A gravity well that attracts nearby particles.
#[derive(Component)]
pub struct GravityWell {
    /// Absolute gravity strength in world units per second² at unit distance.
    pub strength: f32,
}

/// Minimum distance to avoid singularity in gravity calculation.
const MIN_DIST: f32 = 20.0;

/// Apply gravitational attraction from every well to every particle.
///
/// Uses `accel = (delta.normalize() * well.strength) / dist²`, clamped at MIN_DIST.
pub fn apply_gravity(
    wells: Query<(&Transform, &GravityWell)>,
    mut particles: Query<(&mut Particle, &Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut particle, p_transform) in &mut particles {
        let p_pos = p_transform.translation.truncate();

        for (w_transform, well) in &wells {
            let w_pos = w_transform.translation.truncate();
            let delta = w_pos - p_pos;
            let dist = delta.length().max(MIN_DIST);
            let accel = delta.normalize() * well.strength / (dist * dist);
            particle.velocity += accel * dt;
        }
    }
}
