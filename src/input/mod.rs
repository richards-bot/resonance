use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::audio::scale::PENTATONIC_FREQS;
use crate::physics::gravity::GravityWell;
use crate::physics::particles::{spawn_particles, Particle};

/// Tag for a gravity well currently being dragged by the mouse.
#[derive(Component)]
struct Dragging;

/// Z depth at which new gravity wells are placed when left-clicking.
///
/// Adjusted by scrolling away from existing wells.
#[derive(Resource)]
struct PlacementDepth(f32);

/// Bevy plugin for mouse and keyboard input.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlacementDepth(0.0))
            .add_systems(
                Update,
                (
                    keyboard_input,
                    mouse_input,
                    drag_wells,
                    scroll_well_gravity,
                ),
            );
    }
}

/// Handle keyboard controls:
/// - Space: spawn 20 particles
/// - C: clear particles
/// - R: reset everything
fn keyboard_input(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keys: Res<ButtonInput<KeyCode>>,
    particles: Query<Entity, With<Particle>>,
    wells: Query<Entity, With<GravityWell>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let half = if let Ok(win) = window.get_single() {
        Vec2::new(win.width() * 0.5, win.height() * 0.5)
    } else {
        Vec2::new(640.0, 360.0)
    };

    if keys.just_pressed(KeyCode::Space) {
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
        for entity in &wells {
            commands.entity(entity).despawn();
        }
    }
}

/// Cast a ray from the camera through the cursor and intersect with the plane z = depth.
///
/// Returns `None` if the ray is parallel to the plane.
fn ray_to_plane_z(camera: &Camera, cam_transform: &GlobalTransform, cursor: Vec2, depth: f32) -> Option<Vec3> {
    let ray = camera.viewport_to_world(cam_transform, cursor).ok()?;
    if ray.direction.z.abs() < f32::EPSILON {
        return None;
    }
    let t = (depth - ray.origin.z) / ray.direction.z;
    Some(ray.origin + *ray.direction * t)
}

/// Handle left/right mouse clicks for well creation and removal.
fn mouse_input(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    wells: Query<(Entity, &Transform), With<GravityWell>>,
    placement_depth: Res<PlacementDepth>,
    mut camera_pan_orbit: Query<&mut PanOrbitCamera>,
) {
    let Ok(win) = window.get_single() else { return };
    let Some(cursor) = win.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_q.get_single() else { return };

    let Some(world_pos) = ray_to_plane_z(camera, cam_transform, cursor, placement_depth.0) else {
        return;
    };

    // Right click: remove well near cursor position (at placement depth)
    if mouse.just_pressed(MouseButton::Right) {
        for (entity, transform) in &wells {
            if transform.translation.distance(world_pos) < 50.0 {
                commands.entity(entity).despawn();
                return;
            }
        }
    }

    // Left click: start drag on existing well, or spawn new well
    if mouse.just_pressed(MouseButton::Left) {
        for (entity, transform) in &wells {
            if transform.translation.distance(world_pos) < 50.0 {
                commands.entity(entity).insert(Dragging);
                if let Ok(mut pan_orbit) = camera_pan_orbit.get_single_mut() {
                    pan_orbit.enabled = false;
                }
                return;
            }
        }

        // Spawn a new well at the current placement depth
        commands.spawn((
            GravityWell { strength: 3000.0, influence_radius: 0.0 },
            Transform::from_xyz(world_pos.x, world_pos.y, placement_depth.0),
            GlobalTransform::default(),
            Visibility::Visible,
        ));
    }

    // Release drag
    if mouse.just_released(MouseButton::Left) {
        for (entity, _) in &wells {
            commands.entity(entity).remove::<Dragging>();
        }
        if let Ok(mut pan_orbit) = camera_pan_orbit.get_single_mut() {
            pan_orbit.enabled = true;
        }
    }
}

/// Move dragged gravity wells to follow the cursor, keeping their current z depth.
fn drag_wells(
    mut wells: Query<&mut Transform, (With<GravityWell>, With<Dragging>)>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(win) = window.get_single() else { return };
    let Some(cursor) = win.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_q.get_single() else { return };

    for mut transform in &mut wells {
        let well_z = transform.translation.z;
        let Some(world_pos) = ray_to_plane_z(camera, cam_transform, cursor, well_z) else {
            continue;
        };
        transform.translation.x = world_pos.x;
        transform.translation.y = world_pos.y;
    }
}

/// Scroll near a well to adjust its gravity strength (or Shift+scroll to resize influence radius).
/// Scroll away from all wells to adjust the placement depth for the next well.
///
/// Strength is clamped to `[0.0, 200_000.0]`, step = scroll_y × 2000.
/// Influence radius is clamped to `[0.0, 1500.0]`, step = scroll_y × 30.
/// Placement depth is clamped to `[-800.0, 800.0]`, step = scroll_y × 20.
fn scroll_well_gravity(
    mut scroll_events: EventReader<MouseWheel>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut wells: Query<(&Transform, &mut GravityWell)>,
    mut placement_depth: ResMut<PlacementDepth>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok(win) = window.get_single() else { return };
    let Some(cursor) = win.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_q.get_single() else { return };

    let Some(world_pos) = ray_to_plane_z(camera, cam_transform, cursor, placement_depth.0) else {
        return;
    };

    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    for event in scroll_events.read() {
        let delta_y = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 16.0,
        };

        let mut near_well = false;
        for (transform, mut well) in &mut wells {
            if transform.translation.distance(world_pos) < 60.0 {
                if shift {
                    well.influence_radius = (well.influence_radius + delta_y * 30.0).clamp(0.0, 1500.0);
                } else {
                    well.strength = (well.strength + delta_y * 5000.0).clamp(0.0, 200_000.0);
                }
                near_well = true;
            }
        }

        if !near_well {
            placement_depth.0 = (placement_depth.0 + delta_y * 20.0).clamp(-800.0, 800.0);
        }
    }
}
