use bevy::prelude::*;

use crate::physics::particles::Particle;

/// Draw motion trails and specular highlights using Bevy's immediate-mode Gizmos API.
pub fn draw_trails(query: Query<(&Particle, &Transform)>, mut gizmos: Gizmos) {
    for (particle, transform) in &query {
        let base_color = particle.color;
        let trail_len = particle.trail.len();
        if trail_len >= 2 {
            for i in 0..trail_len.saturating_sub(1) {
                let alpha = (i as f32 / trail_len as f32) * 0.4;
                let color = base_color.with_alpha(alpha);
                let from = particle.trail[i];
                let to = particle.trail[i + 1];
                gizmos.line_2d(from, to, color);
            }
        }

        // Specular highlight — small circle offset toward upper-right for 3D sphere illusion
        let pos = transform.translation.truncate();
        let highlight_center = pos + Vec2::new(particle.radius * 0.3, particle.radius * 0.3);
        let highlight_radius = particle.radius * 0.25;
        gizmos.circle_2d(highlight_center, highlight_radius, Color::srgba(1.0, 1.0, 1.0, 0.5));
    }
}
