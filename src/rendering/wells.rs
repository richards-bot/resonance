use bevy::math::Isometry3d;
use bevy::prelude::*;

use crate::physics::gravity::GravityWell;

/// Pulse period for the well ring animation in seconds.
const PULSE_PERIOD: f32 = 1.5;

/// Animate gravity well wireframe rings — pulsing radius drawn via gizmos.
///
/// Three rings at orthogonal orientations give a sphere-like wireframe appearance.
/// Outer rings scale with well strength so stronger wells have a bigger visual footprint.
pub fn animate_wells(
    query: Query<(&Transform, &GravityWell)>,
    mut gizmos: Gizmos,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (transform, well) in &query {
        let pos = transform.translation;
        let pulse = ((t / PULSE_PERIOD) * std::f32::consts::TAU).sin() * 0.5 + 0.5;

        // Inner wireframe — three rings at orthogonal planes
        let inner_r = 12.0 + pulse * 4.0;
        let color1 = Color::srgba(0.4, 0.8, 1.0, 0.7);
        gizmos.circle(Isometry3d::new(pos, Quat::IDENTITY), inner_r, color1);
        gizmos.circle(
            Isometry3d::new(pos, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            inner_r,
            color1,
        );
        gizmos.circle(
            Isometry3d::new(pos, Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
            inner_r,
            color1,
        );

        // Outer influence rings — scale with well strength
        let base_outer = 20.0 + (well.strength / 200_000.0_f32).sqrt() * 60.0;
        let outer_r = base_outer + pulse * 8.0;
        let color2 = Color::srgba(0.4, 0.8, 1.0, 0.2);
        gizmos.circle(Isometry3d::new(pos, Quat::IDENTITY), outer_r, color2);
        gizmos.circle(
            Isometry3d::new(pos, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            outer_r,
            color2,
        );
        gizmos.circle(
            Isometry3d::new(pos, Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
            outer_r,
            color2,
        );

        // Influence radius boundary ring — shown only when a finite radius is set
        if well.influence_radius > 0.0 {
            let ir = well.influence_radius;
            let color3 = Color::srgba(1.0, 0.5, 0.2, 0.15);
            gizmos.circle(Isometry3d::new(pos, Quat::IDENTITY), ir, color3);
            gizmos.circle(
                Isometry3d::new(pos, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                ir,
                color3,
            );
            gizmos.circle(
                Isometry3d::new(pos, Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                ir,
                color3,
            );
        }
    }
}
