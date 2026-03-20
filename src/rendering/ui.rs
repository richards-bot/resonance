use bevy::prelude::*;

use crate::physics::particles::{GravityStrength, Particle};

/// Marker for the particle count text node.
#[derive(Component)]
pub struct ParticleCountText;

/// Marker for the gravity strength text node.
#[derive(Component)]
pub struct GravityStrengthText;

/// Spawn the HUD text nodes.
pub fn setup_hud(mut commands: Commands) {
    // Root UI node (full screen)
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        })
        .with_children(|parent| {
            // Top-left info panel
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((
                        Text::new("Particles: 0"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                        ParticleCountText,
                    ));
                    col.spawn((
                        Text::new("Gravity: 500"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgba(0.7, 0.9, 1.0, 0.85)),
                        GravityStrengthText,
                    ));
                });

            // Bottom controls hint
            parent.spawn((
                Text::new(
                    "Space: spawn  C: clear  R: reset  +/-: gravity  LClick: well  RClick: remove well",
                ),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgba(0.6, 0.6, 0.6, 0.7)),
            ));
        });
}

/// Update the particle count display each frame.
pub fn update_hud(
    particles: Query<(), With<Particle>>,
    mut count_text: Query<&mut Text, (With<ParticleCountText>, Without<GravityStrengthText>)>,
    mut gravity_text: Query<&mut Text, (With<GravityStrengthText>, Without<ParticleCountText>)>,
    gravity: Res<GravityStrength>,
) {
    let count = particles.iter().count();
    if let Ok(mut text) = count_text.get_single_mut() {
        **text = format!("Particles: {}", count);
    }
    if gravity.is_changed() {
        if let Ok(mut text) = gravity_text.get_single_mut() {
            **text = format!("Gravity: {:.0}", gravity.0);
        }
    }
}
