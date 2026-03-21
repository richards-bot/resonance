use std::collections::VecDeque;

use bevy::prelude::*;
use rand::Rng;

/// A simulated particle with position, velocity, radius, and mass.
#[derive(Component)]
pub struct Particle {
    /// Linear velocity in world units per second.
    pub velocity: Vec2,
    /// Radius used for collision and rendering.
    pub radius: f32,
    /// Mass — affects collision response and audio pitch scaling.
    pub mass: f32,
    /// Base frequency (Hz) assigned at spawn from pentatonic scale.
    pub frequency: f32,
    /// Trail history — ring buffer of recent world positions (oldest first).
    pub trail: VecDeque<Vec2>,
    /// Particle color derived from frequency at spawn.
    pub color: Color,
}

/// Maximum number of trail positions retained per particle.
const TRAIL_LEN: usize = 8;

/// Boundary half-extents — particles outside this area are removed.
const BOUNDS: f32 = 900.0;

/// Spawn `count` particles at random positions with random velocities.
///
/// Uses `Mesh2d` + `MeshMaterial2d<ColorMaterial>` with a `Circle` mesh for round rendering.
pub fn spawn_particles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    count: usize,
    window_half: Vec2,
    frequencies: &[f32],
) {
    let mut rng = rand::thread_rng();
    for _ in 0..count {
        let x = rng.gen_range(-window_half.x * 0.8..window_half.x * 0.8);
        let y = rng.gen_range(-window_half.y * 0.8..window_half.y * 0.8);
        let vx = rng.gen_range(-120.0..120.0_f32);
        let vy = rng.gen_range(-120.0..120.0_f32);
        let radius = rng.gen_range(4.0..10.0_f32);
        let mass = radius * radius; // mass proportional to area
        let freq_idx = rng.gen_range(0..frequencies.len());
        let frequency = frequencies[freq_idx];

        // Map frequency to hue: low=blue (240°), high=red (0°)
        let freq_min = 220.0_f32;
        let freq_max = 1760.0_f32;
        let t = ((frequency - freq_min) / (freq_max - freq_min)).clamp(0.0, 1.0);
        let hue = (1.0 - t) * 240.0;
        let color = Color::hsl(hue, 0.9, 0.65);

        let mesh = meshes.add(Circle::new(radius));
        let material = materials.add(ColorMaterial::from_color(color));

        commands.spawn((
            Particle {
                velocity: Vec2::new(vx, vy),
                radius,
                mass,
                frequency,
                trail: VecDeque::with_capacity(TRAIL_LEN + 1),
                color,
            },
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(x, y, 1.0),
        ));
    }
}

/// Semi-implicit Euler integration: apply velocity, apply drag, update trail.
pub fn integrate_particles(
    mut query: Query<(&mut Particle, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    const DRAG: f32 = 0.985;

    for (mut particle, mut transform) in &mut query {
        // Record trail position before moving — O(1) push_back + pop_front
        let pos = transform.translation.truncate();
        particle.trail.push_back(pos);
        if particle.trail.len() > TRAIL_LEN {
            particle.trail.pop_front();
        }

        // Integrate position
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Linear drag
        particle.velocity *= DRAG;
    }
}

/// Remove particles that stray too far from the origin.
pub fn despawn_escaped_particles(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Particle>>,
) {
    for (entity, transform) in &query {
        let pos = transform.translation.truncate();
        if pos.x.abs() > BOUNDS || pos.y.abs() > BOUNDS {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawn the 2D camera.
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
