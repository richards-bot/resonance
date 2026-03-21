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

/// Pause state — when `true`, all physics systems stop updating.
///
/// Body placement and camera controls remain active while paused.
#[derive(Resource, Default, PartialEq)]
pub struct Paused(pub bool);

/// Run condition: returns `true` when the simulation is not paused.
fn not_paused(paused: Res<Paused>) -> bool {
    !paused.0
}

/// Bevy plugin that wires up all physics systems.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationMode>()
            .init_resource::<Paused>()
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
                    .chain()
                    .run_if(not_paused),
            );
    }
}
