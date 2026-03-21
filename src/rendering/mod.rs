use bevy::prelude::*;

pub mod particles;
pub mod star;
pub mod ui;

/// Bevy plugin that sets up all rendering systems.
pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.08)))
            .add_systems(Startup, ui::setup_hud)
            .add_systems(
                Update,
                (
                    particles::draw_trails,
                    particles::draw_body_trails,
                    star::animate_star,
                    ui::update_hud,
                ),
            );
    }
}
