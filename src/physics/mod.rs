use bevy::prelude::*;

pub mod bodies;
pub mod collision;
pub mod gravity;
pub mod particles;

/// Simulation mode — controls whether merging is active and timestep scaling.
#[derive(Resource, Default, PartialEq, Clone, Copy)]
pub enum SimulationMode {
    /// Normal mode: body merging enabled, full timestep.
    #[default]
    Normal,
    /// Three-body mode: merging disabled, half timestep for better accuracy.
    ThreeBody,
}

/// Bevy plugin that wires up all physics systems.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationMode>()
            .add_event::<collision::CollisionEvent>()
            .add_systems(Startup, particles::setup_camera)
            .add_systems(
                Update,
                (
                    gravity::apply_particle_gravity,
                    gravity::apply_n_body_gravity,
                    gravity::merge_bodies.run_if(resource_equals(SimulationMode::Normal)),
                    collision::detect_collisions,
                    particles::integrate_particles,
                    particles::despawn_escaped_particles,
                )
                    .chain(),
            );
    }
}
