use bevy::math::Isometry3d;
use bevy::prelude::*;

use crate::physics::bodies::Star;

/// Pulse period for the star ring animation in seconds.
const PULSE_PERIOD: f32 = 2.0;

/// Animate the star with pulsing wireframe rings drawn via gizmos.
///
/// Three concentric rings at two radii give a glowing corona effect.
pub fn animate_star(query: Query<&Transform, With<Star>>, mut gizmos: Gizmos, time: Res<Time>) {
    let t = time.elapsed_secs();

    for transform in &query {
        let pos = transform.translation;
        let pulse = ((t / PULSE_PERIOD) * std::f32::consts::TAU).sin() * 0.5 + 0.5;

        // Inner corona rings
        let inner_r = 70.0 + pulse * 10.0;
        let color1 = Color::srgba(1.0, 0.9, 0.3, 0.6);
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

        // Outer corona rings
        let outer_r = 90.0 + pulse * 20.0;
        let color2 = Color::srgba(1.0, 0.7, 0.1, 0.25);
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
    }
}
