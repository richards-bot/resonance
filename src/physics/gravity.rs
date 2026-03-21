use bevy::prelude::*;

use super::bodies::{Moon, Planet, Star, Trail, Velocity, G};
use super::particles::Particle;

/// Minimum distance clamp to prevent singularity blow-up on close approach.
const MIN_DIST: f32 = 40.0;

/// Apply Newtonian n-body gravity between all bodies.
///
/// Every body (Star, Planet, Moon, Particle) pulls every other body.
/// The Star has no `Velocity` component — its position is fixed at the origin.
/// Uses a snapshot-then-update approach to avoid Bevy B0001 query conflicts.
pub fn apply_n_body_gravity(
    star_q: Query<(&Transform, &Star)>,
    planet_q: Query<(Entity, &Transform, &Planet)>,
    moon_q: Query<(Entity, &Transform, &Moon)>,
    mut vel_q: Query<&mut Velocity>,
    mut particle_q: Query<(&mut Particle, &Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    // Immutable snapshot of all body positions and masses
    let stars: Vec<(Vec3, f32)> =
        star_q.iter().map(|(t, s)| (t.translation, s.mass)).collect();
    let planets: Vec<(Entity, Vec3, f32)> =
        planet_q.iter().map(|(e, t, p)| (e, t.translation, p.mass)).collect();
    let moons: Vec<(Entity, Vec3, f32)> =
        moon_q.iter().map(|(e, t, m)| (e, t.translation, m.mass)).collect();

    // Update planet velocities
    for (entity, pos, _) in &planets {
        let mut accel = Vec3::ZERO;
        for (s_pos, s_mass) in &stars {
            let delta = *s_pos - *pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * s_mass / (dist * dist);
        }
        for (other_e, other_pos, other_mass) in &planets {
            if other_e == entity {
                continue;
            }
            let delta = *other_pos - *pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * other_mass / (dist * dist);
        }
        for (_, m_pos, m_mass) in &moons {
            let delta = *m_pos - *pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * m_mass / (dist * dist);
        }
        if let Ok(mut vel) = vel_q.get_mut(*entity) {
            vel.0 += accel * dt;
        }
    }

    // Update moon velocities
    for (entity, pos, _) in &moons {
        let mut accel = Vec3::ZERO;
        for (s_pos, s_mass) in &stars {
            let delta = *s_pos - *pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * s_mass / (dist * dist);
        }
        for (_, p_pos, p_mass) in &planets {
            let delta = *p_pos - *pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * p_mass / (dist * dist);
        }
        for (other_e, other_pos, other_mass) in &moons {
            if other_e == entity {
                continue;
            }
            let delta = *other_pos - *pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * other_mass / (dist * dist);
        }
        if let Ok(mut vel) = vel_q.get_mut(*entity) {
            vel.0 += accel * dt;
        }
    }

    // Update particle velocities (attracted by star, planets, and moons)
    for (mut particle, transform) in &mut particle_q {
        let pos = transform.translation;
        let mut accel = Vec3::ZERO;
        for (s_pos, s_mass) in &stars {
            let delta = *s_pos - pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * s_mass / (dist * dist);
        }
        for (_, p_pos, p_mass) in &planets {
            let delta = *p_pos - pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * p_mass / (dist * dist);
        }
        for (_, m_pos, m_mass) in &moons {
            let delta = *m_pos - pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * m_mass / (dist * dist);
        }
        particle.velocity += accel * dt;
    }
}

/// Integrate `Velocity` into `Transform` for Planet and Moon entities.
///
/// Also pushes the pre-move position into the body's `Trail` ring buffer.
pub fn integrate_bodies(
    mut planet_q: Query<
        (&mut Transform, &Velocity, Option<&mut Trail>),
        (With<Planet>, Without<Moon>),
    >,
    mut moon_q: Query<
        (&mut Transform, &Velocity, Option<&mut Trail>),
        (With<Moon>, Without<Planet>),
    >,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut transform, vel, trail_opt) in &mut planet_q {
        if let Some(mut trail) = trail_opt {
            trail.positions.push_back(transform.translation);
            if trail.positions.len() > trail.max_len {
                trail.positions.pop_front();
            }
        }
        transform.translation += vel.0 * dt;
    }

    for (mut transform, vel, trail_opt) in &mut moon_q {
        if let Some(mut trail) = trail_opt {
            trail.positions.push_back(transform.translation);
            if trail.positions.len() > trail.max_len {
                trail.positions.pop_front();
            }
        }
        transform.translation += vel.0 * dt;
    }
}
