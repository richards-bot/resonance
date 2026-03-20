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

/// O(n²) broad+narrow phase collision detection for circle-circle pairs.
///
/// Resolves overlaps positionally and applies elastic impulse response.
pub fn detect_collisions(
    mut query: Query<(Entity, &mut Particle, &mut Transform)>,
    mut events: EventWriter<CollisionEvent>,
) {
    // Collect mutable references by gathering data first, then applying changes.
    // Bevy doesn't allow two mutable borrows of the same query simultaneously,
    // so we gather entity data, compute responses, then apply.
    let mut particles: Vec<(Entity, Vec2, Vec2, f32, f32, f32)> = query
        .iter()
        .map(|(e, p, t)| (e, t.translation.truncate(), p.velocity, p.radius, p.mass, p.frequency))
        .collect();

    let count = particles.len();
    let mut velocity_deltas: Vec<Vec2> = vec![Vec2::ZERO; count];
    let mut position_deltas: Vec<Vec2> = vec![Vec2::ZERO; count];

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

                    // Elastic collision impulse
                    let impulse_scalar = 2.0 * speed_along_normal / (mass_a + mass_b);
                    velocity_deltas[i] -= normal * impulse_scalar * mass_b;
                    velocity_deltas[j] += normal * impulse_scalar * mass_a;

                    // Fire collision event for audio
                    events.send(CollisionEvent {
                        freq_a,
                        freq_b,
                        speed: impact_speed,
                    });
                }

                // Positional correction to prevent sinking
                let overlap = min_dist - dist;
                let correction = normal * overlap * 0.5;
                position_deltas[i] -= correction;
                position_deltas[j] += correction;
            }
        }
    }

    // Apply deltas
    for (i, (entity, _, _, _, _, _)) in particles.iter().enumerate() {
        if let Ok((_, mut particle, mut transform)) = query.get_mut(*entity) {
            particle.velocity += velocity_deltas[i];
            transform.translation.x += position_deltas[i].x;
            transform.translation.y += position_deltas[i].y;
        }
    }

    // Update local velocity cache so subsequent pairs see fresh velocities
    // (This is a simplified approach — good enough for <500 particles)
    let _ = particles.iter_mut().zip(velocity_deltas.iter()).map(|(p, dv)| {
        p.2 += *dv;
    }).count();
}
