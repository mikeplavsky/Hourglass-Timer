use crate::resources::{COLOR_PALETTE, ColorMode, HourglassConfig};
use crate::ui::ColorRowMarker;
use bevy::prelude::*;
use rand::Rng;

pub struct ColorPanelPlugin;

impl Plugin for ColorPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_color_buttons)
            .add_systems(
                Update,
                (
                    handle_color_button_clicks,
                    handle_random_color_button,
                    handle_rainbow_color_button,
                    update_rainbow_color,
                ),
            );
    }
}

#[derive(Component)]
struct ColorButton {
    color: Color,
}

#[derive(Component)]
struct RandomColorButton;

#[derive(Component)]
struct RainbowColorButton;

fn spawn_color_buttons(mut commands: Commands, query: Query<Entity, With<ColorRowMarker>>) {
    // Find the color row container
    if let Ok(panel_entity) = query.single() {
        commands.entity(panel_entity).with_children(|parent| {
            // Add color buttons in horizontal layout
            for (i, &color) in COLOR_PALETTE.iter().enumerate() {
                parent.spawn((
                    Name::new(format!("Color Button {i}")),
                    ColorButton { color },
                    Button,
                    Node {
                        width: Val::Px(20.0),
                        height: Val::Px(20.0),
                        margin: UiRect::horizontal(Val::Px(2.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_shrink: 0.0, // Prevent shrinking
                        ..default()
                    },
                    BackgroundColor(color),
                    BorderColor(Color::WHITE),
                ));
            }

            // Add Random Color Button
            parent.spawn((
                Name::new("Random Color Button"),
                RandomColorButton,
                Button,
                Node {
                    width: Val::Px(40.0),
                    height: Val::Px(20.0),
                    margin: UiRect::horizontal(Val::Px(2.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_shrink: 0.0,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                BorderColor(Color::WHITE),
                Text::new("RND"),
                TextColor(Color::WHITE),
            ));

            // Add Rainbow Color Button
            parent.spawn((
                Name::new("Rainbow Color Button"),
                RainbowColorButton,
                Button,
                Node {
                    width: Val::Px(50.0),
                    height: Val::Px(20.0),
                    margin: UiRect::horizontal(Val::Px(2.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_shrink: 0.0,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.5, 0.2, 0.7)),
                BorderColor(Color::WHITE),
                Text::new("RGB"),
                TextColor(Color::WHITE),
            ));
        });
    }
}

fn handle_color_button_clicks(
    mut interaction_query: Query<
        (&Interaction, &ColorButton, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut config: ResMut<HourglassConfig>,
) {
    for (interaction, color_button, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                config.color = color_button.color;
                config.color_mode = ColorMode::Static;
                *border_color = BorderColor(Color::srgb(0.0, 1.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
            }
            Interaction::None => {
                *border_color = BorderColor(Color::WHITE);
            }
        }
    }
}

fn handle_random_color_button(
    mut interaction_query: Query<
        (&Interaction, &mut BorderColor),
        (Changed<Interaction>, With<RandomColorButton>),
    >,
    mut config: ResMut<HourglassConfig>,
) {
    for (interaction, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let mut rng = rand::thread_rng();
                config.color = Color::srgb(
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                );
                config.color_mode = ColorMode::Random;
                *border_color = BorderColor(Color::srgb(0.0, 1.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
            }
            Interaction::None => {
                *border_color = BorderColor(Color::WHITE);
            }
        }
    }
}

fn handle_rainbow_color_button(
    mut interaction_query: Query<
        (&Interaction, &mut BorderColor),
        (Changed<Interaction>, With<RainbowColorButton>),
    >,
    mut config: ResMut<HourglassConfig>,
) {
    for (interaction, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                config.color_mode = ColorMode::Rainbow;
                *border_color = BorderColor(Color::srgb(0.0, 1.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
            }
            Interaction::None => {
                *border_color = BorderColor(Color::WHITE);
            }
        }
    }
}

fn update_rainbow_color(time: Res<Time>, mut config: ResMut<HourglassConfig>) {
    if config.color_mode == ColorMode::Rainbow {
        // Cycle through hue over time (0-360 degrees)
        let hue = (time.elapsed_secs() * 60.0) % 360.0; // Complete cycle every 6 seconds

        // Convert HSL to RGB (saturation = 1.0, lightness = 0.5 for vibrant colors)
        config.color = hsl_to_rgb(hue, 1.0, 0.5);
    }
}

// Helper function to convert HSL to RGB
fn hsl_to_rgb(hue: f32, saturation: f32, lightness: f32) -> Color {
    let hue = hue / 360.0; // Normalize to 0-1
    let c = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let x = c * (1.0 - ((hue * 6.0) % 2.0 - 1.0).abs());
    let m = lightness - c / 2.0;

    let (r, g, b) = if hue < 1.0 / 6.0 {
        (c, x, 0.0)
    } else if hue < 2.0 / 6.0 {
        (x, c, 0.0)
    } else if hue < 3.0 / 6.0 {
        (0.0, c, x)
    } else if hue < 4.0 / 6.0 {
        (0.0, x, c)
    } else if hue < 5.0 / 6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color::srgb(r + m, g + m, b + m)
}
