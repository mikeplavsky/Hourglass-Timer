use crate::resources::{
    AppearanceStateChanged, COLOR_PALETTE, ColorMode, HourglassConfig, PendingFlip,
};
use crate::timer::{TimerCommand, TimerSet};
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
                )
                    .in_set(TimerSet::Input),
            )
            .add_systems(Update, (update_rainbow_color,));
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

            // Add Random Color Button with multi-colored squares pattern
            parent
                .spawn((
                    Name::new("Random Color Button"),
                    RandomColorButton,
                    Button,
                    Node {
                        width: Val::Px(32.0),
                        height: Val::Px(20.0),
                        margin: UiRect::horizontal(Val::Px(2.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        flex_shrink: 0.0,
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    BorderColor(Color::WHITE),
                ))
                .with_children(|parent| {
                    // Left question mark
                    parent.spawn((
                        Text::new("?"),
                        TextColor(Color::WHITE),
                        Node {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ));

                    // Create a 2x2 grid of colored squares to represent randomness
                    parent
                        .spawn((Node {
                            display: Display::Grid,
                            grid_template_columns: vec![GridTrack::px(8.0), GridTrack::px(8.0)],
                            grid_template_rows: vec![GridTrack::px(8.0), GridTrack::px(8.0)],
                            column_gap: Val::Px(1.0),
                            row_gap: Val::Px(1.0),
                            ..default()
                        },))
                        .with_children(|grid| {
                            // Four small colored squares
                            let colors = [
                                Color::srgb(1.0, 0.2, 0.2), // Red
                                Color::srgb(0.2, 1.0, 0.2), // Green
                                Color::srgb(0.2, 0.2, 1.0), // Blue
                                Color::srgb(1.0, 1.0, 0.2), // Yellow
                            ];

                            for color in colors {
                                grid.spawn((
                                    Node {
                                        width: Val::Px(8.0),
                                        height: Val::Px(8.0),
                                        ..default()
                                    },
                                    BackgroundColor(color),
                                ));
                            }
                        });

                    // Right question mark
                    parent.spawn((
                        TextColor(Color::WHITE),
                        Text::new(" "),
                        Node {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ));
                });

            // Add Rainbow Color Button with gradient stripes
            parent
                .spawn((
                    Name::new("Rainbow Color Button"),
                    RainbowColorButton,
                    Button,
                    Node {
                        width: Val::Px(36.0),
                        height: Val::Px(20.0),
                        margin: UiRect::horizontal(Val::Px(2.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_shrink: 0.0,
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                    BorderColor(Color::WHITE),
                ))
                .with_children(|parent| {
                    // Create rainbow stripes with more colors for seamless transition
                    let rainbow_colors = [
                        Color::srgb(1.0, 0.0, 0.0),  // Red
                        Color::srgb(1.0, 0.25, 0.0), // Red-Orange
                        Color::srgb(1.0, 0.5, 0.0),  // Orange
                        Color::srgb(1.0, 0.75, 0.0), // Orange-Yellow
                        Color::srgb(1.0, 1.0, 0.0),  // Yellow
                        Color::srgb(0.75, 1.0, 0.0), // Yellow-Green
                        Color::srgb(0.5, 1.0, 0.0),  // Lime Green
                        Color::srgb(0.25, 1.0, 0.0), // Light Green
                        Color::srgb(0.0, 1.0, 0.0),  // Green
                        Color::srgb(0.0, 1.0, 0.5),  // Green-Cyan
                        Color::srgb(0.0, 1.0, 1.0),  // Cyan
                        Color::srgb(0.0, 0.5, 1.0),  // Light Blue
                        Color::srgb(0.0, 0.0, 1.0),  // Blue
                        Color::srgb(0.25, 0.0, 1.0), // Blue-Violet
                        Color::srgb(0.5, 0.0, 1.0),  // Purple
                        Color::srgb(0.75, 0.0, 1.0), // Violet
                    ];

                    for color in rainbow_colors {
                        parent.spawn((
                            Node {
                                width: Val::Px(2.0), // Narrower stripes to fit more colors
                                height: Val::Px(18.0),
                                ..default()
                            },
                            BackgroundColor(color),
                        ));
                    }
                });
        });
    }
}

fn handle_color_button_clicks(
    mut interaction_query: Query<
        (&Interaction, &ColorButton, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut config: ResMut<HourglassConfig>,
    mut pending_flip: ResMut<PendingFlip>,
    mut timer_commands: EventWriter<TimerCommand>,
    mut appearance_changed: EventWriter<AppearanceStateChanged>,
) {
    for (interaction, color_button, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                config.color = color_button.color;
                config.color_mode = ColorMode::Static;
                // Changing the color starts the countdown over from full and flips
                timer_commands.write(TimerCommand::Restart);
                appearance_changed.write_default();
                pending_flip.0 = true;
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

/// Squared Euclidean distance between two colors in sRGB space.
/// Squared to avoid an unnecessary `sqrt` when only comparing against a threshold.
fn color_dist_sq(a: Srgba, b: Srgba) -> f32 {
    let dr = a.red - b.red;
    let dg = a.green - b.green;
    let db = a.blue - b.blue;
    dr * dr + dg * dg + db * db
}

/// Pick a random sRGB color that is at least `min_dist_sq` (squared RGB
/// distance) away from `current`, re-rolling until the constraint is met.
fn pick_distinct_color(current: Srgba, min_dist_sq: f32, rng: &mut impl Rng) -> Srgba {
    let mut new_color = Srgba::rgb(
        rng.gen_range(0.0..1.0),
        rng.gen_range(0.0..1.0),
        rng.gen_range(0.0..1.0),
    );
    // Re-roll until the new color is far enough from the current one
    while color_dist_sq(new_color, current) < min_dist_sq {
        new_color = Srgba::rgb(
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
        );
    }
    new_color
}

fn handle_random_color_button(
    mut interaction_query: Query<
        (&Interaction, &mut BorderColor),
        (Changed<Interaction>, With<RandomColorButton>),
    >,
    mut config: ResMut<HourglassConfig>,
    mut pending_flip: ResMut<PendingFlip>,
    mut timer_commands: EventWriter<TimerCommand>,
    mut appearance_changed: EventWriter<AppearanceStateChanged>,
) {
    for (interaction, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let mut rng = rand::thread_rng();
                // Minimum squared RGB distance so the new color is noticeably
                // different from the current one (max possible distance ~1.732).
                const MIN_COLOR_DIST_SQ: f32 = 0.3 * 0.3;
                let current = config.color.to_srgba();
                let new_color = pick_distinct_color(current, MIN_COLOR_DIST_SQ, &mut rng);
                config.color = new_color.into();
                config.color_mode = ColorMode::Random;
                // Changing the color starts the countdown over from full and flips
                timer_commands.write(TimerCommand::Restart);
                appearance_changed.write_default();
                pending_flip.0 = true;
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
    mut pending_flip: ResMut<PendingFlip>,
    mut timer_commands: EventWriter<TimerCommand>,
    mut appearance_changed: EventWriter<AppearanceStateChanged>,
) {
    for (interaction, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                config.color_mode = ColorMode::Rainbow;
                // Activating rainbow starts the countdown over from full and flips
                // (the continuous cycling in update_rainbow_color does not restart it)
                timer_commands.write(TimerCommand::Restart);
                appearance_changed.write_default();
                pending_flip.0 = true;
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
        let hue = rainbow_hue(time.elapsed_secs());

        // Convert HSL to RGB (saturation = 1.0, lightness = 0.5 for vibrant colors)
        config.color = hsl_to_rgb(hue, 1.0, 0.5);
    }
}

/// Hue (in degrees, 0-360) for the rainbow animation at the given elapsed
/// time. Completes one full cycle every 6 seconds.
fn rainbow_hue(elapsed_secs: f32) -> f32 {
    (elapsed_secs * 60.0) % 360.0
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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    /// Extract (r, g, b) from a `Color` for comparison.
    fn rgb(color: Color) -> (f32, f32, f32) {
        let s = color.to_srgba();
        (s.red, s.green, s.blue)
    }

    fn assert_rgb(actual: Color, expected: (f32, f32, f32)) {
        let (r, g, b) = rgb(actual);
        assert_abs_diff_eq!(r, expected.0, epsilon = 1e-5);
        assert_abs_diff_eq!(g, expected.1, epsilon = 1e-5);
        assert_abs_diff_eq!(b, expected.2, epsilon = 1e-5);
    }

    #[test]
    fn color_dist_sq_identical_is_zero() {
        let c = Srgba::rgb(0.5, 0.5, 0.5);
        assert_abs_diff_eq!(color_dist_sq(c, c), 0.0, epsilon = 1e-6);
    }

    #[test]
    fn color_dist_sq_black_to_white_is_three() {
        let black = Srgba::rgb(0.0, 0.0, 0.0);
        let white = Srgba::rgb(1.0, 1.0, 1.0);
        assert_abs_diff_eq!(color_dist_sq(black, white), 3.0, epsilon = 1e-6);
    }

    #[test]
    fn color_dist_sq_is_symmetric() {
        let a = Srgba::rgb(0.1, 0.7, 0.3);
        let b = Srgba::rgb(0.8, 0.2, 0.9);
        assert_abs_diff_eq!(color_dist_sq(a, b), color_dist_sq(b, a), epsilon = 1e-6);
    }

    #[test]
    fn color_dist_sq_matches_threshold() {
        // Single channel differs by 0.3 -> squared distance 0.09, the exact
        // threshold used by pick_distinct_color (0.3 * 0.3).
        let a = Srgba::rgb(0.2, 0.4, 0.6);
        let b = Srgba::rgb(0.5, 0.4, 0.6);
        assert_abs_diff_eq!(color_dist_sq(a, b), 0.09, epsilon = 1e-6);
    }

    #[test]
    fn pick_distinct_color_respects_min_distance() {
        let current = Srgba::rgb(0.5, 0.5, 0.5);
        let min_dist_sq = 0.3 * 0.3;
        for seed in 0..20 {
            let mut rng = StdRng::seed_from_u64(seed);
            let new_color = pick_distinct_color(current, min_dist_sq, &mut rng);
            assert!(
                color_dist_sq(new_color, current) >= min_dist_sq,
                "seed {seed}: color too close to current"
            );
        }
    }

    #[test]
    fn pick_distinct_color_channels_in_range() {
        let current = Srgba::rgb(0.5, 0.5, 0.5);
        for seed in 0..20 {
            let mut rng = StdRng::seed_from_u64(seed);
            let c = pick_distinct_color(current, 0.09, &mut rng);
            for ch in [c.red, c.green, c.blue] {
                assert!(
                    (0.0..1.0).contains(&ch),
                    "seed {seed}: channel {ch} out of range"
                );
            }
        }
    }

    #[test]
    fn pick_distinct_color_zero_threshold_is_deterministic() {
        // With min_dist_sq = 0 the constraint is trivially satisfied (the loop
        // never runs), so the same seed always yields the same first roll.
        let current = Srgba::rgb(0.5, 0.5, 0.5);
        let mut rng_a = StdRng::seed_from_u64(123);
        let mut rng_b = StdRng::seed_from_u64(123);
        let a = pick_distinct_color(current, 0.0, &mut rng_a);
        let b = pick_distinct_color(current, 0.0, &mut rng_b);
        assert_eq!(a, b);
    }

    #[test]
    fn rainbow_hue_cycles() {
        assert_abs_diff_eq!(rainbow_hue(0.0), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rainbow_hue(3.0), 180.0, epsilon = 1e-4);
        // Full cycle wraps back to 0 after 6 seconds.
        assert_abs_diff_eq!(rainbow_hue(6.0), 0.0, epsilon = 1e-4);
        assert_abs_diff_eq!(rainbow_hue(7.5), 90.0, epsilon = 1e-4);
    }

    #[test]
    fn rainbow_hue_stays_in_range() {
        for i in 0..200 {
            let elapsed = i as f32 * 0.137;
            let hue = rainbow_hue(elapsed);
            assert!((0.0..360.0).contains(&hue), "hue {hue} out of range");
        }
    }

    #[test]
    fn hsl_to_rgb_primary_and_secondary_hues() {
        assert_rgb(hsl_to_rgb(0.0, 1.0, 0.5), (1.0, 0.0, 0.0)); // red
        assert_rgb(hsl_to_rgb(60.0, 1.0, 0.5), (1.0, 1.0, 0.0)); // yellow
        assert_rgb(hsl_to_rgb(180.0, 1.0, 0.5), (0.0, 1.0, 1.0)); // cyan
        assert_rgb(hsl_to_rgb(240.0, 1.0, 0.5), (0.0, 0.0, 1.0)); // blue
        assert_rgb(hsl_to_rgb(300.0, 1.0, 0.5), (1.0, 0.0, 1.0)); // magenta
    }

    #[test]
    fn hsl_to_rgb_zero_saturation_is_gray() {
        assert_rgb(hsl_to_rgb(123.0, 0.0, 0.5), (0.5, 0.5, 0.5));
    }

    #[test]
    fn hsl_to_rgb_lightness_extremes() {
        assert_rgb(hsl_to_rgb(0.0, 1.0, 0.0), (0.0, 0.0, 0.0)); // black
        assert_rgb(hsl_to_rgb(0.0, 1.0, 1.0), (1.0, 1.0, 1.0)); // white
    }
}
