use bevy::prelude::*;

use super::particles::Particle;

/// Radius threshold above which a particle acts as a mini gravity well.
const LARGE_RADIUS: f32 = 45.0;

/// A gravity well that attracts nearby particles.
#[derive(Component)]
pub struct GravityWell {
    /// Absolute gravity strength in world units per second² at unit distance.
    pub strength: f32,
    /// Maximum influence radius in world units. `0.0` means infinite influence.
    pub influence_radius: f32,
}

/// Global gravitational constant — scales all wells up to pixel-friendly magnitudes.
const G: f32 = 80_000.0;

/// Minimum distance clamp to prevent singularity blow-up on close approach.
const MIN_DIST: f32 = 50.0;

/// Apply gravitational attraction from large particles (radius > 45) to all other particles.
///
/// Large particles act as mini gravity wells with `effective_strength = mass × 0.15`.
/// We snapshot large-particle positions first to avoid borrow conflicts.
pub fn apply_particle_gravity(
    source_query: Query<(Entity, &Particle, &Transform)>,
    mut target_query: Query<(Entity, &mut Particle, &Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    // Collect large-particle data to avoid aliasing issues with the mutable borrow below
    let sources: Vec<(Entity, Vec3, f32)> = source_query
        .iter()
        .filter(|(_, p, _)| p.radius > LARGE_RADIUS)
        .map(|(e, p, t)| (e, t.translation, p.mass * 0.15))
        .collect();

    if sources.is_empty() {
        return;
    }

    for (target_entity, mut particle, p_transform) in &mut target_query {
        let p_pos = p_transform.translation;
        let effective_mass = particle.mass.min(200.0);

        for (src_entity, src_pos, strength) in &sources {
            // Do not apply a particle's own gravity to itself
            if *src_entity == target_entity {
                continue;
            }
            let delta = src_pos - p_pos;
            let dist = delta.length().max(MIN_DIST);
            let accel = delta.normalize() * G * strength / (dist * dist * effective_mass);
            particle.velocity += accel * dt;
        }
    }
}

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
            // Cap effective mass so large particles aren't over-dampened
            let effective_mass = particle.mass.min(200.0);
            let accel = delta.normalize() * G * well.strength / (dist * dist * effective_mass);
            particle.velocity += accel * dt;
        }
    }
}
