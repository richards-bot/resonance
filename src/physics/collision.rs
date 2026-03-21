use bevy::prelude::*;

use super::particles::Particle;

/// Fired whenever two particles collide.
#[derive(Event)]
#[allow(dead_code)]
pub struct CollisionEvent {
    /// Frequency of particle A.
    pub freq_a: f32,
    /// Frequency of particle B.
    pub freq_b: f32,
    /// Relative speed at impact — used to set audio volume.
    pub speed: f32,
}

/// O(n²) broad+narrow phase collision detection for sphere-sphere pairs.
///
/// Resolves overlaps positionally and applies elastic impulse response.
pub fn detect_collisions(
    mut query: Query<(Entity, &mut Particle, &mut Transform)>,
    mut events: EventWriter<CollisionEvent>,
) {
    // Gather snapshot — Bevy prevents two simultaneous mutable query borrows.
    let particles: Vec<(Entity, Vec3, Vec3, f32, f32, f32)> = query
        .iter()
        .map(|(e, p, t)| (e, t.translation, p.velocity, p.radius, p.mass, p.frequency))
        .collect();

    let count = particles.len();
    let mut velocity_deltas: Vec<Vec3> = vec![Vec3::ZERO; count];
    let mut position_deltas: Vec<Vec3> = vec![Vec3::ZERO; count];

    for i in 0..count {
        for j in (i + 1)..count {
            let (_, pos_a, vel_a, rad_a, mass_a, freq_a) = particles[i];
            let (_, pos_b, vel_b, rad_b, mass_b, freq_b) = particles[j];

            let delta = pos_b - pos_a;
            let dist = delta.length();
            let min_dist = rad_a + rad_b;

            if dist < min_dist && dist > f32::EPSILON {
                let normal = delta / dist;
                let relative_vel = vel_a - vel_b;
                let speed_along_normal = relative_vel.dot(normal);

                // Only resolve if objects are approaching
                if speed_along_normal > 0.0 {
                    let impact_speed = speed_along_normal.abs();

                    // Elastic impulse magnitude: 2 * dot(rel_vel,n) * m_a*m_b / (m_a+m_b)
                    // Δv_a = -impulse/m_a * n,  Δv_b = +impulse/m_b * n
                    let impulse = 2.0 * speed_along_normal * mass_a * mass_b / (mass_a + mass_b);
                    velocity_deltas[i] -= normal * impulse / mass_a;
                    velocity_deltas[j] += normal * impulse / mass_b;

                    events.send(CollisionEvent {
                        freq_a,
                        freq_b,
                        speed: impact_speed,
                    });
                }

                // Positional correction — split overlap evenly, weighted by mass
                let overlap = min_dist - dist;
                let total_mass = mass_a + mass_b;
                position_deltas[i] -= normal * overlap * (mass_b / total_mass);
                position_deltas[j] += normal * overlap * (mass_a / total_mass);
            }
        }
    }

    // Apply deltas back into the ECS
    for (i, (entity, _, _, _, _, _)) in particles.iter().enumerate() {
        if let Ok((_, mut particle, mut transform)) = query.get_mut(*entity) {
            particle.velocity += velocity_deltas[i];
            transform.translation += position_deltas[i];
        }
    }
}
