use crate::resources::TimerState;
use crate::ui::{BottomTimerMarker, TimerPanelVisible};
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;

pub struct TimerPanelPlugin;

impl Plugin for TimerPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_timer_controls)
            .add_systems(
                Update,
                (
                    handle_timer_buttons,
                    update_time_display,
                    handle_control_buttons,
                    handle_toggle_button,
                    update_timer_panel_visibility,
                ),
            );
    }
}

#[derive(Component)]
struct TimeAdjustButton {
    adjustment: f32, // in seconds
}

#[derive(Component)]
struct TimeDisplay;

#[derive(Component)]
struct StartButton;

#[derive(Component)]
struct PauseButton;

#[derive(Component)]
struct ResetButton;

#[derive(Component)]
struct ToggleButton;

#[derive(Component)]
struct TimerControlsContainer;

fn spawn_timer_controls(mut commands: Commands, query: Query<Entity, With<BottomTimerMarker>>) {
    // Find the bottom timer container
    if let Ok(panel_entity) = query.single() {
        commands.entity(panel_entity).with_children(|parent| {
            // Toggle button (always visible)
            parent
                .spawn((
                    ToggleButton,
                    Button,
                    Node {
                        width: Val::Px(130.0),
                        height: Val::Px(30.0),
                        margin: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                    BorderColor(Color::WHITE),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Timer Controls"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Timer controls container (initially hidden)
            parent
                .spawn((
                    TimerControlsContainer,
                    Node {
                        width: Val::Percent(100.0),
                        display: Display::None, // Start hidden
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                ))
                .with_children(|parent| {
                    spawn_timer_controls_content(parent);
                });
        });
    }
}

fn spawn_timer_controls_content(parent: &mut RelatedSpawnerCommands<ChildOf>) {
    // Time controls row
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            // Time adjustment buttons (negative)
            let negative_adjustments = [
                ("-1h", -3600.0),
                ("-15m", -900.0),
                ("-5m", -300.0),
                ("-1m", -60.0),
                ("-15s", -15.0),
                ("-5s", -5.0),
                ("-1s", -1.0),
            ];

            for (label, adjustment) in negative_adjustments {
                parent
                    .spawn((
                        TimeAdjustButton { adjustment },
                        Button,
                        Node {
                            width: Val::Px(50.0),
                            height: Val::Px(40.0),
                            margin: UiRect::horizontal(Val::Px(3.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        BorderColor(Color::srgb(0.5, 0.5, 0.5)),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            }

            // Time display
            parent.spawn((
                TimeDisplay,
                Text::new("00:03:00"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::horizontal(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Time adjustment buttons (positive)
            let positive_adjustments = [
                ("+1s", 1.0),
                ("+5s", 5.0),
                ("+15s", 15.0),
                ("+1m", 60.0),
                ("+5m", 300.0),
                ("+15m", 900.0),
                ("+1h", 3600.0),
            ];

            for (label, adjustment) in positive_adjustments {
                parent
                    .spawn((
                        TimeAdjustButton { adjustment },
                        Button,
                        Node {
                            width: Val::Px(50.0),
                            height: Val::Px(40.0),
                            margin: UiRect::horizontal(Val::Px(3.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        BorderColor(Color::srgb(0.5, 0.5, 0.5)),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            }
        });

    // Control buttons row
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    StartButton,
                    Button,
                    Node {
                        width: Val::Px(60.0),
                        height: Val::Px(40.0),
                        margin: UiRect::horizontal(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.7, 0.2)),
                    BorderColor(Color::WHITE),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Start"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            parent
                .spawn((
                    PauseButton,
                    Button,
                    Node {
                        width: Val::Px(60.0),
                        height: Val::Px(40.0),
                        margin: UiRect::horizontal(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.7, 0.7, 0.2)),
                    BorderColor(Color::WHITE),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Pause"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            parent
                .spawn((
                    ResetButton,
                    Button,
                    Node {
                        width: Val::Px(60.0),
                        height: Val::Px(40.0),
                        margin: UiRect::horizontal(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.7, 0.2, 0.2)),
                    BorderColor(Color::WHITE),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Reset"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn handle_timer_buttons(
    mut interaction_query: Query<
        (&Interaction, &TimeAdjustButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut timer_state: ResMut<TimerState>,
) {
    for (interaction, button, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                timer_state.add_time(button.adjustment);
                *bg_color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
            }
        }
    }
}

fn handle_control_buttons(
    mut start_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<StartButton>,
            Without<PauseButton>,
            Without<ResetButton>,
        ),
    >,
    mut pause_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<PauseButton>,
            Without<StartButton>,
            Without<ResetButton>,
        ),
    >,
    mut reset_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<ResetButton>,
            Without<StartButton>,
            Without<PauseButton>,
        ),
    >,
    mut timer_state: ResMut<TimerState>,
) {
    // Handle Start button
    for (interaction, mut bg_color) in &mut start_query {
        match *interaction {
            Interaction::Pressed => {
                if !timer_state.is_running {
                    timer_state.is_running = true;
                }
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.8, 0.3));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.75, 0.25));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.7, 0.2));
            }
        }
    }

    // Handle Pause button
    for (interaction, mut bg_color) in &mut pause_query {
        match *interaction {
            Interaction::Pressed => {
                timer_state.is_running = false;
                *bg_color = BackgroundColor(Color::srgb(0.8, 0.8, 0.3));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.75, 0.75, 0.25));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.7, 0.7, 0.2));
            }
        }
    }

    // Handle Reset button
    for (interaction, mut bg_color) in &mut reset_query {
        match *interaction {
            Interaction::Pressed => {
                timer_state.reset();
                *bg_color = BackgroundColor(Color::srgb(0.8, 0.3, 0.3));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.75, 0.25, 0.25));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.7, 0.2, 0.2));
            }
        }
    }
}

fn handle_toggle_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ToggleButton>),
    >,
    mut panel_visible: ResMut<TimerPanelVisible>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                panel_visible.0 = !panel_visible.0;
                *bg_color = BackgroundColor(Color::srgb(0.6, 0.6, 0.6));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
            }
        }
    }
}

fn update_timer_panel_visibility(
    panel_visible: Res<TimerPanelVisible>,
    mut query: Query<&mut Node, With<TimerControlsContainer>>,
) {
    if panel_visible.is_changed() {
        for mut node in &mut query {
            node.display = if panel_visible.0 {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

fn update_time_display(
    timer_state: Res<TimerState>,
    panel_visible: Res<TimerPanelVisible>,
    mut query: Query<&mut Text, With<TimeDisplay>>,
) {
    // Only update time display if panel is visible
    if panel_visible.0 {
        for mut text in &mut query {
            **text = timer_state.format_time();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Headless `App` tests for the timer-panel button/display systems. Each
    // spawns its UI entity in `Startup` (so it is flushed before `Update` runs
    // on the same tick) and presses a button by spawning `Interaction::Pressed`
    // explicitly — `Button`'s required `Interaction` defaults to `None`, which
    // would otherwise route every handler through its no-op arm. `Changed<_>`
    // and `Res::is_changed()` both fire on the first tick for freshly inserted
    // components/resources, so a single `app.update()` is enough.

    /// Build a headless app with the given `TimerState` and one pressed button
    /// carrying the marker bundle `B`. The caller adds the system under test and
    /// calls `app.update()`.
    fn pressed_button_app<B: Bundle>(timer_state: TimerState, marker: B) -> App {
        let mut app = App::new();
        app.insert_resource(timer_state);
        app.world_mut().spawn((marker, Button, Interaction::Pressed));
        app
    }

    // --- handle_timer_buttons ---------------------------------------------

    #[test]
    fn time_adjust_button_press_adds_time() {
        let mut app = pressed_button_app(
            TimerState {
                duration: 180.0,
                remaining: 180.0,
                is_running: false,
            },
            TimeAdjustButton { adjustment: 60.0 },
        );
        app.add_systems(Update, handle_timer_buttons);
        app.update();
        let ts = app.world().resource::<TimerState>();
        assert_eq!(ts.duration, 240.0);
        assert_eq!(ts.remaining, 240.0);
    }

    #[test]
    fn time_adjust_button_negative_subtracts_time() {
        let mut app = pressed_button_app(
            TimerState {
                duration: 180.0,
                remaining: 180.0,
                is_running: false,
            },
            TimeAdjustButton { adjustment: -60.0 },
        );
        app.add_systems(Update, handle_timer_buttons);
        app.update();
        let ts = app.world().resource::<TimerState>();
        assert_eq!(ts.duration, 120.0);
        assert_eq!(ts.remaining, 120.0);
    }

    // --- handle_control_buttons (one app per button: outcomes conflict) ---

    #[test]
    fn start_button_sets_running() {
        let mut app = pressed_button_app(
            TimerState {
                duration: 180.0,
                remaining: 180.0,
                is_running: false,
            },
            StartButton,
        );
        app.add_systems(Update, handle_control_buttons);
        app.update();
        assert!(app.world().resource::<TimerState>().is_running);
    }

    #[test]
    fn pause_button_clears_running() {
        let mut app = pressed_button_app(
            TimerState {
                duration: 180.0,
                remaining: 90.0,
                is_running: true,
            },
            PauseButton,
        );
        app.add_systems(Update, handle_control_buttons);
        app.update();
        assert!(!app.world().resource::<TimerState>().is_running);
    }

    #[test]
    fn reset_button_restores_and_stops() {
        let mut app = pressed_button_app(
            TimerState {
                duration: 180.0,
                remaining: 5.0,
                is_running: true,
            },
            ResetButton,
        );
        app.add_systems(Update, handle_control_buttons);
        app.update();
        let ts = app.world().resource::<TimerState>();
        assert_eq!(ts.remaining, 180.0);
        assert!(!ts.is_running);
    }

    // --- handle_toggle_button ---------------------------------------------

    /// Press the toggle button once against a `TimerPanelVisible(initial)`.
    fn toggle_app(initial: bool) -> App {
        let mut app = App::new();
        app.insert_resource(TimerPanelVisible(initial));
        app.world_mut()
            .spawn((ToggleButton, Button, Interaction::Pressed));
        app.add_systems(Update, handle_toggle_button);
        app.update();
        app
    }

    #[test]
    fn toggle_button_flips_visibility_on() {
        let app = toggle_app(false);
        assert!(app.world().resource::<TimerPanelVisible>().0);
    }

    #[test]
    fn toggle_button_flips_visibility_off() {
        let app = toggle_app(true);
        assert!(!app.world().resource::<TimerPanelVisible>().0);
    }

    // --- update_timer_panel_visibility ------------------------------------

    /// Spawn a hidden `TimerControlsContainer`, set panel visibility, run the
    /// visibility system once, and return the resulting `display`. The resource
    /// reads as changed on the first tick, so the guarded body runs.
    fn container_display(visible: bool) -> Display {
        let mut app = App::new();
        app.insert_resource(TimerPanelVisible(visible));
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                TimerControlsContainer,
                Node {
                    display: Display::None,
                    ..default()
                },
            ));
        });
        app.add_systems(Update, update_timer_panel_visibility);
        app.update();
        let mut query = app
            .world_mut()
            .query_filtered::<&Node, With<TimerControlsContainer>>();
        query.single(app.world()).unwrap().display
    }

    #[test]
    fn visible_panel_shows_container() {
        assert_eq!(container_display(true), Display::Flex);
    }

    #[test]
    fn hidden_panel_collapses_container() {
        assert_eq!(container_display(false), Display::None);
    }

    // --- update_time_display ----------------------------------------------

    /// Spawn a `TimeDisplay` text, run the display system once, and return its
    /// text. Starts as `"xx"` so the not-visible case is detectable.
    fn time_display_text(visible: bool, remaining: f32) -> String {
        let mut app = App::new();
        app.insert_resource(TimerPanelVisible(visible));
        app.insert_resource(TimerState {
            duration: 180.0,
            remaining,
            is_running: false,
        });
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((TimeDisplay, Text::new("xx")));
        });
        app.add_systems(Update, update_time_display);
        app.update();
        let mut query = app
            .world_mut()
            .query_filtered::<&Text, With<TimeDisplay>>();
        query.single(app.world()).unwrap().0.clone()
    }

    #[test]
    fn time_display_updates_when_panel_visible() {
        // 65s -> 00:01:05.
        assert_eq!(time_display_text(true, 65.0), "00:01:05");
    }

    #[test]
    fn time_display_untouched_when_panel_hidden() {
        assert_eq!(time_display_text(false, 65.0), "xx");
    }
}
