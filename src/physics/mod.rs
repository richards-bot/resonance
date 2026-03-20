use bevy::prelude::*;

pub mod collision;
pub mod gravity;
pub mod particles;

/// Bevy plugin that wires up all physics systems.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(particles::GravityStrength(500.0))
            .add_event::<collision::CollisionEvent>()
            .add_systems(Startup, particles::setup_camera)
            .add_systems(
                Update,
                (
                    gravity::apply_gravity,
                    collision::detect_collisions,
                    particles::integrate_particles,
                    particles::despawn_escaped_particles,
                )
                    .chain(),
            );
    }
}
