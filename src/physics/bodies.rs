use std::collections::VecDeque;

use bevy::prelude::*;
use rand::Rng;

/// Gravitational constant — must match the value used in `gravity.rs`.
pub const G: f32 = 500.0;

/// Velocity component for Planet and Moon entities.
#[derive(Component)]
pub struct Velocity(pub Vec3);

/// Trail component used by Planet and Moon for motion trail rendering.
#[derive(Component)]
pub struct Trail {
    /// Ring buffer of recent world positions (oldest first).
    pub positions: VecDeque<Vec3>,
    /// Display colour of this body's trail.
    pub color: Color,
    /// Maximum number of positions retained.
    pub max_len: usize,
}

impl Trail {
    /// Create an empty trail with the given colour and history length.
    pub fn new(color: Color, max_len: usize) -> Self {
        Self {
            positions: VecDeque::with_capacity(max_len + 1),
            color,
            max_len,
        }
    }
}

/// The star fixed at the origin — drives gravitational pull on all bodies.
#[derive(Component)]
pub struct Star {
    /// Gravitational mass of the star. Scroll near it to adjust.
    pub mass: f32,
}

/// A planet orbiting the star.
#[derive(Component)]
pub struct Planet {
    /// Gravitational mass derived from radius.
    pub mass: f32,
    /// Human-readable name assigned at spawn.
    pub name: String,
}

/// A moon orbiting a specific planet.
#[derive(Component)]
pub struct Moon {
    /// Gravitational mass derived from radius.
    pub mass: f32,
    /// The planet entity this moon orbits.
    pub parent_planet: Entity,
}

/// Five-colour palette shared by Particle and Planet.
const PLANET_PALETTE: [Color; 5] = [
    Color::srgb(0.93, 0.36, 0.32), // coral red
    Color::srgb(0.97, 0.64, 0.10), // amber
    Color::srgb(0.16, 0.80, 0.73), // teal
    Color::srgb(0.63, 0.34, 0.88), // violet
    Color::srgb(0.46, 0.82, 0.17), // lime
];

const PLANET_NAMES: [&str; 8] =
    ["Aether", "Borea", "Calos", "Dusk", "Ember", "Frost", "Gale", "Haze"];

/// Spawn the star at the world origin (position ignored — always placed at 0,0,0).
pub fn spawn_star(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let radius = 60.0_f32;
    let mesh = meshes.add(Sphere::new(radius));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.95, 0.6),
        emissive: LinearRgba::new(3.0, 2.85, 1.8, 1.0),
        metallic: 0.0,
        perceptual_roughness: 0.2,
        ..default()
    });
    commands.spawn((
        Star { mass: 1_000_000.0 },
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

/// Spawn a planet at `position` with a computed circular orbital velocity around the star.
///
/// The orbital velocity direction is perpendicular to the star→planet radius vector in the XY plane.
pub fn spawn_planet(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    star_mass: f32,
) {
    let mut rng = rand::thread_rng();
    let radius = rng.gen_range(12.0..28.0_f32);
    let mass = radius * radius * radius * 0.01;
    let color = PLANET_PALETTE[rng.gen_range(0..PLANET_PALETTE.len())];
    let name = PLANET_NAMES[rng.gen_range(0..PLANET_NAMES.len())].to_string();

    // Circular orbital velocity: v = sqrt(G * M / r)
    let pos2d = Vec2::new(position.x, position.y);
    let r = pos2d.length().max(1.0);
    let v_orbital = (G * star_mass / r).sqrt();
    // Perpendicular direction (counter-clockwise) in XY plane
    let dir = Vec2::new(-pos2d.y, pos2d.x).normalize();
    let velocity = Vec3::new(dir.x * v_orbital, dir.y * v_orbital, 0.0);

    let mesh = meshes.add(Sphere::new(radius));
    let material = materials.add(StandardMaterial {
        base_color: color,
        metallic: 0.1,
        perceptual_roughness: 0.5,
        ..default()
    });

    commands.spawn((
        Planet { mass, name },
        Velocity(velocity),
        Trail::new(color, 40),
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(position.x, position.y, 0.0),
    ));
}

/// Spawn a moon at `position` orbiting `planet_entity`.
///
/// The orbital velocity is computed relative to the parent planet and then added to
/// the planet's current velocity so the moon moves with it.
pub fn spawn_moon(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    planet_entity: Entity,
    planet_pos: Vec3,
    planet_mass: f32,
    planet_velocity: Vec3,
) {
    let mut rng = rand::thread_rng();
    let radius = rng.gen_range(4.0..10.0_f32);
    let mass = radius * radius;
    let color = Color::srgb(0.75, 0.75, 0.78);

    // Orbital velocity around parent planet
    let delta = Vec2::new(position.x - planet_pos.x, position.y - planet_pos.y);
    let r = delta.length().max(1.0);
    let v_orbital = (G * planet_mass / r).sqrt();
    let dir = Vec2::new(-delta.y, delta.x).normalize();
    let velocity = planet_velocity + Vec3::new(dir.x * v_orbital, dir.y * v_orbital, 0.0);

    let mesh = meshes.add(Sphere::new(radius));
    let material = materials.add(StandardMaterial {
        base_color: color,
        metallic: 0.05,
        perceptual_roughness: 0.6,
        ..default()
    });

    commands.spawn((
        Moon { mass, parent_planet: planet_entity },
        Velocity(velocity),
        Trail::new(color, 30),
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(position.x, position.y, planet_pos.z),
    ));
}
