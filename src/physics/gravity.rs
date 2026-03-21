use bevy::prelude::*;

use super::particles::Particle;

/// A gravity well that attracts nearby particles.
#[derive(Component)]
pub struct GravityWell {
    /// Absolute gravity strength in world units per second² at unit distance.
    pub strength: f32,
    /// Maximum influence radius in world units. `0.0` means infinite influence.
    pub influence_radius: f32,
}

/// Global gravitational constant — scales all wells up to pixel-friendly magnitudes.
const G: f32 = 2_000.0;

/// Minimum distance clamp to prevent singularity blow-up on close approach.
const MIN_DIST: f32 = 35.0;

/// Apply gravitational attraction from every well to every particle.
///
/// Smaller (less massive) particles accelerate faster — acceleration is
/// inversely proportional to mass: `accel = G × well.strength / (dist² × mass)`.
/// Since mass ∝ radius², a particle half the radius accelerates 4× faster.
pub fn apply_gravity(
    wells: Query<(&Transform, &GravityWell)>,
    mut particles: Query<(&mut Particle, &Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut particle, p_transform) in &mut particles {
        let p_pos = p_transform.translation;

        for (w_transform, well) in &wells {
            let w_pos = w_transform.translation;
            let delta = w_pos - p_pos;
            let dist = delta.length().max(MIN_DIST);
            if well.influence_radius > 0.0 && dist > well.influence_radius {
                continue;
            }
            // G scales up the force to compensate for pixel-scale distances
            // Dividing by mass means smaller particles pull in faster
            let accel = delta.normalize() * G * well.strength / (dist * dist * particle.mass);
            particle.velocity += accel * dt;
        }
    }
}
