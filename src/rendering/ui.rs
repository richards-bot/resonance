use bevy::prelude::*;

use crate::input::PlacementMode;
use crate::physics::bodies::{Moon, Planet, Star};
use crate::physics::particles::Particle;

/// Marker for the dynamic info text node (mode + body counts).
#[derive(Component)]
pub struct InfoText;

/// Spawn the HUD text nodes.
pub fn setup_hud(mut commands: Commands) {
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
            // Top-left dynamic info: mode and body counts
            parent.spawn((
                Text::new("Mode: NONE | Star: 0 | Planets: 0 | Moons: 0 | Debris: 0"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                InfoText,
            ));

            // Bottom: scroll hint + controls
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((
                        Text::new("Scroll near star = mass"),
                        TextFont { font_size: 13.0, ..default() },
                        TextColor(Color::srgba(0.7, 0.7, 0.4, 0.7)),
                    ));
                    col.spawn((
                        Text::new(
                            "1=Star  2=Planet  3=Moon  Esc=None  Space=Debris  C=Clear debris  R=Reset  RClick=Remove body",
                        ),
                        TextFont { font_size: 13.0, ..default() },
                        TextColor(Color::srgba(0.6, 0.6, 0.6, 0.7)),
                    ));
                });
        });
}

/// Update the dynamic HUD info text each frame.
pub fn update_hud(
    mode: Res<PlacementMode>,
    particles: Query<(), With<Particle>>,
    planets: Query<(), With<Planet>>,
    moons: Query<(), With<Moon>>,
    stars: Query<(), With<Star>>,
    mut info_text: Query<&mut Text, With<InfoText>>,
) {
    let mode_str = match *mode {
        PlacementMode::None => "NONE",
        PlacementMode::Star => "STAR",
        PlacementMode::Planet => "PLANET",
        PlacementMode::Moon => "MOON",
    };

    let star_count = stars.iter().count();
    let planet_count = planets.iter().count();
    let moon_count = moons.iter().count();
    let debris_count = particles.iter().count();

    if let Ok(mut text) = info_text.get_single_mut() {
        **text = format!(
            "Mode: {}  |  Star: {}  Planets: {}  Moons: {}  Debris: {}",
            mode_str, star_count, planet_count, moon_count, debris_count
        );
    }
}
