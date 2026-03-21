use bevy::prelude::*;

use super::bodies::{spawn_star_at, Moon, Planet, Star, Trail, Velocity, G};
use super::particles::Particle;
use super::SimulationMode;

/// Minimum distance clamp for debris particles.
const MIN_DIST: f32 = 40.0;
/// Minimum distance clamp for star-star interactions (point masses).
const MIN_DIST_STAR: f32 = 8.0;
/// Minimum distance clamp for star-planet and planet-planet interactions.
const MIN_DIST_BODY: f32 = 20.0;

/// Compute gravitational acceleration on body at `pos_i` from all other bodies.
fn compute_accel(pos_i: Vec3, others: &[(Vec3, f32)], min_dist: f32) -> Vec3 {
    let mut accel = Vec3::ZERO;
    for (pos_j, mass_j) in others {
        let delta = *pos_j - pos_i;
        let dist = delta.length().max(min_dist);
        accel += delta.normalize() * G * mass_j / (dist * dist);
    }
    accel
}

/// Apply RK4 N-body gravity integration for Stars, Planets, and Moons.
///
/// Integrates both velocity and position in a single step using 4th-order Runge-Kutta.
/// Each body's acceleration uses a snapshot of all other bodies' starting positions,
/// which is the standard decoupled-RK4 approach for N-body systems.
/// In `ThreeBody` mode the timestep is halved for better numerical accuracy.
pub fn apply_n_body_gravity(
    mut body_q: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            Option<&Star>,
            Option<&Planet>,
            Option<&Moon>,
            Option<&mut Trail>,
        ),
        Or<(With<Star>, With<Planet>, With<Moon>)>,
    >,
    time: Res<Time>,
    sim_mode: Res<SimulationMode>,
) {
    let dt = time.delta_secs()
        * if *sim_mode == SimulationMode::ThreeBody { 0.5 } else { 1.0 };

    // Snapshot all body states (iter() gives read-only access even on a mut query)
    struct BodyState {
        entity: Entity,
        pos: Vec3,
        vel: Vec3,
        mass: f32,
        is_star: bool,
    }

    let snapshots: Vec<BodyState> = body_q
        .iter()
        .filter_map(|(entity, transform, vel, star, planet, moon, _)| {
            let (mass, is_star) = if let Some(s) = star {
                (s.mass, true)
            } else if let Some(p) = planet {
                (p.mass, false)
            } else if let Some(m) = moon {
                (m.mass, false)
            } else {
                return None;
            };
            Some(BodyState {
                entity,
                pos: transform.translation,
                vel: vel.0,
                mass,
                is_star,
            })
        })
        .collect();

    // Compute RK4 updates for each body
    struct Update {
        entity: Entity,
        new_pos: Vec3,
        new_vel: Vec3,
    }

    let updates: Vec<Update> = snapshots
        .iter()
        .map(|body| {
            let others: Vec<(Vec3, f32)> = snapshots
                .iter()
                .filter(|b| b.entity != body.entity)
                .map(|b| (b.pos, b.mass))
                .collect();

            // Stars are treated as point masses; use tighter clamp
            let min_dist = if body.is_star { MIN_DIST_STAR } else { MIN_DIST_BODY };

            let pos = body.pos;
            let vel = body.vel;

            let k1_v = compute_accel(pos, &others, min_dist);
            let k1_x = vel;

            let k2_v = compute_accel(pos + k1_x * dt / 2.0, &others, min_dist);
            let k2_x = vel + k1_v * dt / 2.0;

            let k3_v = compute_accel(pos + k2_x * dt / 2.0, &others, min_dist);
            let k3_x = vel + k2_v * dt / 2.0;

            let k4_v = compute_accel(pos + k3_x * dt, &others, min_dist);
            let k4_x = vel + k3_v * dt;

            let new_vel = vel + (k1_v + 2.0 * k2_v + 2.0 * k3_v + k4_v) * dt / 6.0;
            let new_pos = pos + (k1_x + 2.0 * k2_x + 2.0 * k3_x + k4_x) * dt / 6.0;

            Update { entity: body.entity, new_pos, new_vel }
        })
        .collect();

    // Apply updates and record trail positions before the move
    for update in &updates {
        if let Ok((_, mut transform, mut vel, _, _, _, trail_opt)) =
            body_q.get_mut(update.entity)
        {
            if let Some(mut trail) = trail_opt {
                trail.positions.push_back(transform.translation);
                if trail.positions.len() > trail.max_len {
                    trail.positions.pop_front();
                }
            }
            transform.translation = update.new_pos;
            vel.0 = update.new_vel;
        }
    }
}

/// Apply gravitational acceleration from bodies to particles (Euler integration).
///
/// Runs as a separate system before `apply_n_body_gravity` to avoid Transform
/// access conflicts (bodies need &mut Transform, particles need &Transform).
pub fn apply_particle_gravity(
    star_q: Query<(&Transform, &Star)>,
    planet_q: Query<(&Transform, &Planet)>,
    moon_q: Query<(&Transform, &Moon)>,
    mut particle_q: Query<(&mut Particle, &Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    let body_sources: Vec<(Vec3, f32)> = star_q
        .iter()
        .map(|(t, s)| (t.translation, s.mass))
        .chain(planet_q.iter().map(|(t, p)| (t.translation, p.mass)))
        .chain(moon_q.iter().map(|(t, m)| (t.translation, m.mass)))
        .collect();

    for (mut particle, transform) in &mut particle_q {
        let pos = transform.translation;
        let mut accel = Vec3::ZERO;
        for (body_pos, body_mass) in &body_sources {
            let delta = *body_pos - pos;
            let dist = delta.length().max(MIN_DIST);
            accel += delta.normalize() * G * body_mass / (dist * dist);
        }
        particle.velocity += accel * dt;
    }
}

/// Merge two bodies that have overlapped, conserving linear momentum.
///
/// On each frame, finds the first pair of bodies whose surfaces overlap
/// (distance < sum of radii) and replaces them with a single merged body.
/// Uses perfectly inelastic collision: momentum is conserved, kinetic energy is not.
pub fn merge_bodies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    stars: Query<(Entity, &Transform, &Velocity, &Star)>,
    planets: Query<(Entity, &Transform, &Velocity, &Planet)>,
    moons: Query<(Entity, &Transform, &Velocity, &Moon)>,
) {
    #[derive(Clone, Copy, PartialEq)]
    enum MergeKind {
        Star,
        Planet,
        Moon,
    }

    struct BodySnap {
        entity: Entity,
        pos: Vec3,
        vel: Vec3,
        mass: f32,
        radius: f32,
        kind: MergeKind,
        moon_parent: Option<Entity>,
    }

    let mut bodies: Vec<BodySnap> = Vec::new();

    for (e, t, v, s) in &stars {
        // Stars are point masses for physics; use a small collision radius
        bodies.push(BodySnap {
            entity: e,
            pos: t.translation,
            vel: v.0,
            mass: s.mass,
            radius: 15.0,
            kind: MergeKind::Star,
            moon_parent: None,
        });
    }
    for (e, t, v, p) in &planets {
        let visual_radius = (p.mass * 100.0_f32).cbrt();
        bodies.push(BodySnap {
            entity: e,
            pos: t.translation,
            vel: v.0,
            mass: p.mass,
            radius: visual_radius * 0.5,
            kind: MergeKind::Planet,
            moon_parent: None,
        });
    }
    for (e, t, v, m) in &moons {
        let visual_radius = m.mass.sqrt();
        bodies.push(BodySnap {
            entity: e,
            pos: t.translation,
            vel: v.0,
            mass: m.mass,
            radius: visual_radius * 0.5,
            kind: MergeKind::Moon,
            moon_parent: Some(m.parent_planet),
        });
    }

    // Find first overlapping pair
    let mut merge_pair: Option<(usize, usize)> = None;
    'outer: for i in 0..bodies.len() {
        for j in (i + 1)..bodies.len() {
            if bodies[i].pos.distance(bodies[j].pos) < bodies[i].radius + bodies[j].radius {
                merge_pair = Some((i, j));
                break 'outer;
            }
        }
    }

    let Some((i, j)) = merge_pair else { return };

    let combined_mass = bodies[i].mass + bodies[j].mass;
    let new_vel =
        (bodies[i].vel * bodies[i].mass + bodies[j].vel * bodies[j].mass) / combined_mass;
    let new_pos =
        (bodies[i].pos * bodies[i].mass + bodies[j].pos * bodies[j].mass) / combined_mass;

    commands.entity(bodies[i].entity).despawn();
    commands.entity(bodies[j].entity).despawn();

    let merged_kind = match (bodies[i].kind, bodies[j].kind) {
        (MergeKind::Star, _) | (_, MergeKind::Star) => MergeKind::Star,
        (MergeKind::Planet, _) | (_, MergeKind::Planet) => MergeKind::Planet,
        _ => MergeKind::Moon,
    };

    match merged_kind {
        MergeKind::Star => {
            spawn_star_at(
                &mut commands,
                &mut meshes,
                &mut materials,
                new_pos,
                new_vel,
                combined_mass,
            );
        }
        MergeKind::Planet => {
            let radius = (combined_mass * 100.0_f32).cbrt().min(120.0);
            let color = Color::srgb(1.0, 0.55, 0.1);
            let mesh = meshes.add(Sphere::new(radius));
            let mat = materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.1,
                perceptual_roughness: 0.5,
                ..default()
            });
            commands.spawn((
                Planet { mass: combined_mass, name: "Merged".to_string() },
                Velocity(new_vel),
                Trail::new(color, 40),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_translation(new_pos),
            ));
        }
        MergeKind::Moon => {
            let radius = combined_mass.sqrt().min(30.0);
            let color = Color::srgb(0.82, 0.82, 0.84);
            let mesh = meshes.add(Sphere::new(radius));
            let mat = materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.05,
                perceptual_roughness: 0.6,
                ..default()
            });
            let parent =
                bodies[i].moon_parent.or(bodies[j].moon_parent).unwrap_or(Entity::PLACEHOLDER);
            commands.spawn((
                Moon { mass: combined_mass, parent_planet: parent },
                Velocity(new_vel),
                Trail::new(color, 30),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_translation(new_pos),
            ));
        }
    }
}
