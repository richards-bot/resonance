use bevy::prelude::*;

use super::bodies::{Moon, Planet, Star, Trail, Velocity, G};
use super::particles::Particle;

/// Minimum distance clamp to prevent singularity blow-up on close approach.
const MIN_DIST: f32 = 40.0;

/// Apply Newtonian n-body gravity between all bodies (Stars, Planets, Moons, Particles).
///
/// All body types exert and receive gravitational pulls. Stars are no longer fixed.
/// A spring-tether correction keeps moons near their expected orbital distance.
/// Uses a snapshot-then-update approach to avoid Bevy B0001 query conflicts.
pub fn apply_n_body_gravity(
    star_q: Query<(Entity, &Transform, &Star)>,
    planet_q: Query<(Entity, &Transform, &Planet)>,
    moon_q: Query<(Entity, &Transform, &Moon)>,
    mut vel_q: Query<&mut Velocity>,
    mut particle_q: Query<(&mut Particle, &Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    // Immutable snapshots of all body positions and masses
    let stars: Vec<(Entity, Vec3, f32)> =
        star_q.iter().map(|(e, t, s)| (e, t.translation, s.mass)).collect();
    let planets: Vec<(Entity, Vec3, f32)> =
        planet_q.iter().map(|(e, t, p)| (e, t.translation, p.mass)).collect();
    let moons: Vec<(Entity, Vec3, f32, f32, Entity)> =
        moon_q
            .iter()
            .map(|(e, t, m)| (e, t.translation, m.mass, m.expected_dist, m.parent_planet))
            .collect();

    // Update star velocities (stars pull on each other, on planets, and on moons)
    for (entity, pos, _) in &stars {
        let mut accel = Vec3::ZERO;
        for (other_e, other_pos, other_mass) in &stars {
            if *other_e == *entity {
                continue;
            }
            let delta = *other_pos - *pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * other_mass / (dist * dist);
        }
        for (_, p_pos, p_mass) in &planets {
            let delta = *p_pos - *pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * p_mass / (dist * dist);
        }
        for (_, m_pos, m_mass, _, _) in &moons {
            let delta = *m_pos - *pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * m_mass / (dist * dist);
        }
        if let Ok(mut vel) = vel_q.get_mut(*entity) {
            vel.0 += accel * dt;
        }
    }

    // Update planet velocities
    for (entity, pos, _) in &planets {
        let mut accel = Vec3::ZERO;
        for (_, s_pos, s_mass) in &stars {
            let delta = *s_pos - *pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * s_mass / (dist * dist);
        }
        for (other_e, other_pos, other_mass) in &planets {
            if *other_e == *entity {
                continue;
            }
            let delta = *other_pos - *pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * other_mass / (dist * dist);
        }
        for (_, m_pos, m_mass, _, _) in &moons {
            let delta = *m_pos - *pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * m_mass / (dist * dist);
        }
        if let Ok(mut vel) = vel_q.get_mut(*entity) {
            vel.0 += accel * dt;
        }
    }

    // Update moon velocities (with spring-tether correction toward parent planet)
    for (entity, pos, _, expected_dist, parent_planet) in &moons {
        let mut accel = Vec3::ZERO;
        for (_, s_pos, s_mass) in &stars {
            let delta = *s_pos - *pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * s_mass / (dist * dist);
        }
        for (_, p_pos, p_mass) in &planets {
            let delta = *p_pos - *pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * p_mass / (dist * dist);
        }
        for (other_e, other_pos, other_mass, _, _) in &moons {
            if *other_e == *entity {
                continue;
            }
            let delta = *other_pos - *pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * other_mass / (dist * dist);
        }

        // Spring-tether: gently pull moon back toward expected orbital distance
        if let Some((_, p_pos, _)) = planets.iter().find(|(e, _, _)| e == parent_planet) {
            let tether_delta = *p_pos - *pos;
            let actual_dist = tether_delta.length();
            if actual_dist > expected_dist * 1.5 && actual_dist > f32::EPSILON {
                let extra = (actual_dist - expected_dist) * 0.01;
                accel += tether_delta.normalize() * extra;
            }
        }

        if let Ok(mut vel) = vel_q.get_mut(*entity) {
            vel.0 += accel * dt;
        }
    }

    // Update particle velocities (attracted by all bodies)
    for (mut particle, transform) in &mut particle_q {
        let pos = transform.translation;
        let mut accel = Vec3::ZERO;
        for (_, s_pos, s_mass) in &stars {
            let delta = *s_pos - pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * s_mass / (dist * dist);
        }
        for (_, p_pos, p_mass) in &planets {
            let delta = *p_pos - pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * p_mass / (dist * dist);
        }
        for (_, m_pos, m_mass, _, _) in &moons {
            let delta = *m_pos - pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * m_mass / (dist * dist);
        }
        particle.velocity += accel * dt;
    }
}

/// Integrate `Velocity` into `Transform` for Star, Planet, and Moon entities.
///
/// Also pushes the pre-move position into the body's `Trail` ring buffer.
pub fn integrate_bodies(
    mut body_q: Query<
        (&mut Transform, &Velocity, Option<&mut Trail>),
        Or<(With<Planet>, With<Moon>, With<Star>)>,
    >,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut transform, vel, trail_opt) in &mut body_q {
        if let Some(mut trail) = trail_opt {
            trail.positions.push_back(transform.translation);
            if trail.positions.len() > trail.max_len {
                trail.positions.pop_front();
            }
        }
        transform.translation += vel.0 * dt;
    }
}
