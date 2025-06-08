use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use crate::resources::TimerState;
use crate::ui::{BottomTimerMarker, TimerPanelVisible};

pub struct TimerPanelPlugin;

impl Plugin for TimerPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_timer_controls)
            .add_systems(Update, (
                handle_timer_buttons,
                update_time_display,
                handle_control_buttons,
                handle_toggle_button,
                update_timer_panel_visibility,
            ));
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

fn spawn_timer_controls(
    mut commands: Commands,
    query: Query<Entity, With<BottomTimerMarker>>,
) {
    // Find the bottom timer container
    if let Ok(panel_entity) = query.single() {
        commands.entity(panel_entity).with_children(|parent| {
            // Toggle button (always visible)
            parent.spawn((
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
            )).with_children(|parent| {
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
            parent.spawn((
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
            )).with_children(|parent| {
                spawn_timer_controls_content(parent);
            });
        });
    }
}

fn spawn_timer_controls_content(parent: &mut RelatedSpawnerCommands<ChildOf>) {
    // Time controls row
    parent.spawn((
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
    )).with_children(|parent| {
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
            parent.spawn((
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
            )).with_children(|parent| {
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
            parent.spawn((
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
            )).with_children(|parent| {
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
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
    )).with_children(|parent| {
        parent.spawn((
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
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Start"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

        parent.spawn((
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
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Pause"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

        parent.spawn((
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
        )).with_children(|parent| {
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
        (Changed<Interaction>, With<StartButton>, Without<PauseButton>, Without<ResetButton>),
    >,
    mut pause_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<PauseButton>, Without<StartButton>, Without<ResetButton>),
    >,
    mut reset_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ResetButton>, Without<StartButton>, Without<PauseButton>),
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
