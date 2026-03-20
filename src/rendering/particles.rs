use bevy::prelude::*;

use crate::physics::particles::Particle;

/// Sync sprite size with current particle radius (in case radius changes).
pub fn update_particle_visuals(mut query: Query<(&Particle, &mut Sprite)>) {
    for (particle, mut sprite) in &mut query {
        sprite.custom_size = Some(Vec2::splat(particle.radius * 2.0));
    }
}

/// Draw motion trails using Bevy's immediate-mode Gizmos API.
pub fn draw_trails(query: Query<(&Particle, &Sprite)>, mut gizmos: Gizmos) {
    for (particle, sprite) in &query {
        let base_color = sprite.color;
        let trail_len = particle.trail.len();
        if trail_len < 2 {
            continue;
        }
        for i in 0..trail_len.saturating_sub(1) {
            let alpha = (i as f32 / trail_len as f32) * 0.4;
            let color = base_color.with_alpha(alpha);
            let from = particle.trail[i];
            let to = particle.trail[i + 1];
            gizmos.line_2d(from, to, color);
        }
    }
}
