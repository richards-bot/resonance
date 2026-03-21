use bevy::prelude::*;

use crate::physics::particles::Particle;

/// Draw motion trails using Bevy's immediate-mode Gizmos API.
///
/// Trails are Vec3 positions — PBR handles specular/emissive highlights on the spheres.
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
