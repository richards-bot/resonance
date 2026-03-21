use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::audio::scale::PENTATONIC_FREQS;
use crate::physics::bodies::{spawn_moon, spawn_planet, spawn_star, Moon, Planet, Star, Velocity};
use crate::physics::particles::{spawn_particles, Particle};

/// Current placement mode — determines what a left click spawns.
#[derive(Resource, Default, PartialEq, Debug, Clone, Copy)]
pub enum PlacementMode {
    #[default]
    None,
    Star,
    Planet,
    Moon,
}

/// Bevy plugin for mouse and keyboard input.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlacementMode>().add_systems(
            Update,
            (keyboard_input, mouse_input, scroll_input),
        );
    }
}

/// Cast a ray from the camera through the cursor and intersect with the plane z = depth.
///
/// Returns `None` if the ray is parallel to the plane.
fn ray_to_plane_z(
    camera: &Camera,
    cam_transform: &GlobalTransform,
    cursor: Vec2,
    depth: f32,
) -> Option<Vec3> {
    let ray = camera.viewport_to_world(cam_transform, cursor).ok()?;
    if ray.direction.z.abs() < f32::EPSILON {
        return None;
    }
    let t = (depth - ray.origin.z) / ray.direction.z;
    Some(ray.origin + *ray.direction * t)
}

/// Handle keyboard controls:
/// - `1` → PlacementMode::Star
/// - `2` → PlacementMode::Planet
/// - `3` → PlacementMode::Moon
/// - `Escape` → PlacementMode::None
/// - `Space` → spawn 20 debris particles
/// - `C` → clear debris particles only
/// - `R` → reset everything
fn keyboard_input(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<PlacementMode>,
    particles: Query<Entity, With<Particle>>,
    planets: Query<Entity, With<Planet>>,
    moons: Query<Entity, With<Moon>>,
    stars: Query<Entity, With<Star>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        *mode = PlacementMode::Star;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        *mode = PlacementMode::Planet;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        *mode = PlacementMode::Moon;
    }
    if keys.just_pressed(KeyCode::Escape) {
        *mode = PlacementMode::None;
    }

    if keys.just_pressed(KeyCode::Space) {
        let half = if let Ok(win) = window.get_single() {
            Vec2::new(win.width() * 0.5, win.height() * 0.5)
        } else {
            Vec2::new(640.0, 360.0)
        };
        spawn_particles(&mut commands, &mut meshes, &mut materials, 20, half, PENTATONIC_FREQS);
    }

    if keys.just_pressed(KeyCode::KeyC) {
        for entity in &particles {
            commands.entity(entity).despawn();
        }
    }

    if keys.just_pressed(KeyCode::KeyR) {
        for entity in &particles {
            commands.entity(entity).despawn();
        }
        for entity in &planets {
            commands.entity(entity).despawn();
        }
        for entity in &moons {
            commands.entity(entity).despawn();
        }
        for entity in &stars {
            commands.entity(entity).despawn();
        }
        *mode = PlacementMode::None;
    }
}

/// Handle left/right mouse clicks for body placement and removal.
fn mouse_input(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mode: Res<PlacementMode>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    stars: Query<(Entity, &Transform, &Star)>,
    planets: Query<(Entity, &Transform, &Planet, &Velocity)>,
    moons: Query<(Entity, &Transform), With<Moon>>,
) {
    let Ok(win) = window.get_single() else { return };
    let Some(cursor) = win.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_q.get_single() else { return };
    let Some(world_pos) = ray_to_plane_z(camera, cam_transform, cursor, 0.0) else { return };

    // Right click: despawn any body within 50 units of click
    if mouse.just_pressed(MouseButton::Right) {
        for (entity, transform, _) in &stars {
            if transform.translation.distance(world_pos) < 50.0 {
                commands.entity(entity).despawn();
                return;
            }
        }
        for (entity, transform, _, _) in &planets {
            if transform.translation.distance(world_pos) < 50.0 {
                commands.entity(entity).despawn();
                return;
            }
        }
        for (entity, transform) in &moons {
            if transform.translation.distance(world_pos) < 50.0 {
                commands.entity(entity).despawn();
                return;
            }
        }
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    match *mode {
        PlacementMode::Star => {
            // Despawn existing star, then place new one at origin
            for (entity, _, _) in &stars {
                commands.entity(entity).despawn();
            }
            spawn_star(&mut commands, &mut meshes, &mut materials);
        }
        PlacementMode::Planet => {
            let star_mass =
                stars.iter().next().map(|(_, _, s)| s.mass).unwrap_or(1_000_000.0);
            spawn_planet(&mut commands, &mut meshes, &mut materials, world_pos, star_mass);
        }
        PlacementMode::Moon => {
            // Find the nearest planet within 300 units
            let mut nearest: Option<(Entity, Vec3, f32, Vec3)> = None;
            let mut min_dist = 300.0_f32;
            for (entity, transform, planet, vel) in &planets {
                let dist = transform.translation.distance(world_pos);
                if dist < min_dist {
                    min_dist = dist;
                    nearest = Some((entity, transform.translation, planet.mass, vel.0));
                }
            }
            if let Some((p_entity, p_pos, p_mass, p_vel)) = nearest {
                spawn_moon(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    world_pos,
                    p_entity,
                    p_pos,
                    p_mass,
                    p_vel,
                );
            }
            // If no planet within 300 units, silently do nothing
        }
        PlacementMode::None => {}
    }
}

/// Scroll near the star (within 80 units) to adjust its mass.
///
/// Step = `star.mass × 0.1` per scroll line. Clamped to `[100_000, 10_000_000]`.
fn scroll_input(
    mut scroll_events: EventReader<MouseWheel>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut stars: Query<(&Transform, &mut Star)>,
) {
    let Ok(win) = window.get_single() else { return };
    let Some(cursor) = win.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_q.get_single() else { return };
    let Some(world_pos) = ray_to_plane_z(camera, cam_transform, cursor, 0.0) else { return };

    for event in scroll_events.read() {
        let delta_y = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 16.0,
        };

        for (transform, mut star) in &mut stars {
            if transform.translation.distance(world_pos) < 80.0 {
                let step = star.mass * 0.1 * delta_y;
                star.mass = (star.mass + step).clamp(100_000.0, 10_000_000.0);
            }
        }
    }
}
