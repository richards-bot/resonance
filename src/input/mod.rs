use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::audio::scale::PENTATONIC_FREQS;
use crate::physics::gravity::GravityWell;
use crate::physics::particles::{spawn_particles, Particle};

/// Tag for a gravity well currently being dragged by the mouse.
#[derive(Component)]
struct Dragging;

/// Bevy plugin for mouse and keyboard input.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
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
    mut materials: ResMut<Assets<ColorMaterial>>,
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

/// Handle left/right mouse clicks for well creation and removal.
fn mouse_input(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    wells: Query<(Entity, &Transform), With<GravityWell>>,
) {
    let Ok(win) = window.get_single() else { return };
    let Some(cursor) = win.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_q.get_single() else { return };

    // Convert cursor from viewport space to world space
    let world_pos = match camera.viewport_to_world_2d(cam_transform, cursor) {
        Ok(pos) => pos,
        Err(_) => return,
    };

    // Right click: remove well under cursor
    if mouse.just_pressed(MouseButton::Right) {
        for (entity, transform) in &wells {
            let pos = transform.translation.truncate();
            if pos.distance(world_pos) < 30.0 {
                commands.entity(entity).despawn();
                return;
            }
        }
    }

    // Left click: start drag on existing well, or spawn new well
    if mouse.just_pressed(MouseButton::Left) {
        // Check if clicking near an existing well
        for (entity, transform) in &wells {
            let pos = transform.translation.truncate();
            if pos.distance(world_pos) < 30.0 {
                commands.entity(entity).insert(Dragging);
                return;
            }
        }

        // Spawn a new well with default absolute gravity strength
        commands.spawn((
            GravityWell { strength: 3000.0 },
            Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
            GlobalTransform::default(),
            Visibility::Visible,
        ));
    }

    // Release drag
    if mouse.just_released(MouseButton::Left) {
        for (entity, _) in &wells {
            commands.entity(entity).remove::<Dragging>();
        }
    }
}

/// Move dragged gravity wells to follow the cursor.
fn drag_wells(
    mut wells: Query<&mut Transform, (With<GravityWell>, With<Dragging>)>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(win) = window.get_single() else { return };
    let Some(cursor) = win.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_q.get_single() else { return };

    let world_pos = match camera.viewport_to_world_2d(cam_transform, cursor) {
        Ok(pos) => pos,
        Err(_) => return,
    };

    for mut transform in &mut wells {
        transform.translation.x = world_pos.x;
        transform.translation.y = world_pos.y;
    }
}

/// Adjust gravity strength of a well when the cursor is within 60px and the user scrolls.
///
/// Strength is clamped to `[0.0, 200_000.0]`, step = scroll_y × 2000.
fn scroll_well_gravity(
    mut scroll_events: EventReader<MouseWheel>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut wells: Query<(&Transform, &mut GravityWell)>,
) {
    let Ok(win) = window.get_single() else { return };
    let Some(cursor) = win.cursor_position() else { return };
    let Ok((camera, cam_transform)) = camera_q.get_single() else { return };

    let world_pos = match camera.viewport_to_world_2d(cam_transform, cursor) {
        Ok(pos) => pos,
        Err(_) => return,
    };

    for event in scroll_events.read() {
        let delta_y = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 16.0,
        };

        for (transform, mut well) in &mut wells {
            let pos = transform.translation.truncate();
            if pos.distance(world_pos) < 60.0 {
                well.strength = (well.strength + delta_y * 2000.0).clamp(0.0, 200_000.0);
            }
        }
    }
}
