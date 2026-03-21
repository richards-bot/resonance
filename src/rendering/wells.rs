use bevy::prelude::*;

use crate::physics::gravity::GravityWell;

/// Pulse period for the well ring animation in seconds.
const PULSE_PERIOD: f32 = 1.5;

/// Animate gravity well rings — pulsing radius drawn via gizmos.
///
/// Outer ring scales with well strength so stronger wells have a bigger visual footprint.
pub fn animate_wells(
    query: Query<(&Transform, &GravityWell)>,
    mut gizmos: Gizmos,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (transform, well) in &query {
        let pos = transform.translation.truncate();
        let pulse = ((t / PULSE_PERIOD) * std::f32::consts::TAU).sin() * 0.5 + 0.5;

        // Inner ring
        let inner_r = 12.0 + pulse * 4.0;
        gizmos.circle_2d(pos, inner_r, Color::srgba(0.4, 0.8, 1.0, 0.7));

        // Outer influence ring — scales with well strength
        let base_outer = 20.0 + (well.strength / 200_000.0_f32).sqrt() * 60.0;
        let outer_r = base_outer + pulse * 8.0;
        gizmos.circle_2d(pos, outer_r, Color::srgba(0.4, 0.8, 1.0, 0.2));
    }
}
