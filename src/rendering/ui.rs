use bevy::prelude::*;

use crate::input::{MoonTarget, PlacementMode};
use crate::physics::bodies::{Moon, Planet, Star};
use crate::physics::particles::Particle;
use crate::physics::{Paused, SimulationMode};

/// Marker for the dynamic info text node (mode + body counts).
#[derive(Component)]
pub struct InfoText;

/// Marker for the dynamic controls hint text node.
#[derive(Component)]
pub struct ControlsText;

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
            // Top-left dynamic info: paused state, mode and body counts
            parent.spawn((
                Text::new("Mode: NONE | Star: 0 | Planets: 0 | Moons: 0 | Debris: 0"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
                InfoText,
            ));

            // Bottom: scroll hint + dynamic controls
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
                            "1=Star (max 3)  2=Planet  3=Moon  Esc=None  Space=Debris  C=Clear debris  R=Reset  F=Figure-8  P=Pause  RClick=Remove body",
                        ),
                        TextFont { font_size: 13.0, ..default() },
                        TextColor(Color::srgba(0.6, 0.6, 0.6, 0.7)),
                        ControlsText,
                    ));
                });
        });
}

/// Update the dynamic HUD info and controls hint text each frame.
pub fn update_hud(
    mode: Res<PlacementMode>,
    sim_mode: Res<SimulationMode>,
    paused: Res<Paused>,
    moon_target: Res<MoonTarget>,
    particles: Query<(), With<Particle>>,
    planets: Query<(), With<Planet>>,
    moons: Query<(), With<Moon>>,
    stars: Query<(), With<Star>>,
    mut info_text: Query<&mut Text, (With<InfoText>, Without<ControlsText>)>,
    mut controls_text: Query<&mut Text, (With<ControlsText>, Without<InfoText>)>,
) {
    let mode_str = match *mode {
        PlacementMode::None => "NONE".to_string(),
        PlacementMode::Star => "STAR".to_string(),
        PlacementMode::Planet => "PLANET".to_string(),
        PlacementMode::Moon => {
            if moon_target.planet.is_none() {
                "MOON — Click a planet to target".to_string()
            } else {
                "MOON — Planet targeted — click to place moon".to_string()
            }
        }
    };

    let star_count = stars.iter().count();
    let planet_count = planets.iter().count();
    let moon_count = moons.iter().count();
    let debris_count = particles.iter().count();

    let three_body_notice = if *sim_mode == SimulationMode::ThreeBody {
        "  |  [!] 3-BODY MODE (no merging)"
    } else {
        ""
    };

    if let Ok(mut text) = info_text.get_single_mut() {
        if paused.0 {
            **text = format!(
                "⏸ PAUSED\nMode: {}  |  Star: {}  Planets: {}  Moons: {}  Debris: {}{}",
                mode_str, star_count, planet_count, moon_count, debris_count, three_body_notice
            );
        } else {
            **text = format!(
                "Mode: {}  |  Star: {}  Planets: {}  Moons: {}  Debris: {}{}",
                mode_str, star_count, planet_count, moon_count, debris_count, three_body_notice
            );
        }
    }

    if let Ok(mut text) = controls_text.get_single_mut() {
        let p_hint = if paused.0 { "P=Resume" } else { "P=Pause" };
        **text = format!(
            "1=Star (max 3)  2=Planet  3=Moon  Esc=None  Space=Debris  C=Clear debris  R=Reset  F=Figure-8  {}  RClick=Remove body",
            p_hint
        );
    }
}
