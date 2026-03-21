use bevy::prelude::*;

use crate::physics::bodies::Trail;
use crate::physics::particles::Particle;

/// Draw motion trails for debris particles using Bevy's immediate-mode Gizmos API.
pub fn draw_trails(query: Query<&Particle>, mut gizmos: Gizmos) {
    for particle in &query {
        let base_color = particle.color;
        let trail_len = particle.trail.len();
        if trail_len >= 2 {
            for i in 0..trail_len.saturating_sub(1) {
                let alpha = (i as f32 / trail_len as f32) * 0.4;
                let color = base_color.with_alpha(alpha);
                let from = particle.trail[i];
                let to = particle.trail[i + 1];
                gizmos.line(from, to, color);
            }
        }
    }
}

/// Draw motion trails for Planet and Moon entities using the `Trail` component.
pub fn draw_body_trails(query: Query<&Trail>, mut gizmos: Gizmos) {
    for trail in &query {
        let len = trail.positions.len();
        if len >= 2 {
            for i in 0..len.saturating_sub(1) {
                let alpha = (i as f32 / len as f32) * 0.5;
                let color = trail.color.with_alpha(alpha);
                let from = trail.positions[i];
                let to = trail.positions[i + 1];
                gizmos.line(from, to, color);
            }
        }
    }
}
