use crate::resources::TimerState;
use bevy::prelude::*;

pub struct PauseOverlayPlugin;

impl Plugin for PauseOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_pause_overlay)
            .add_systems(Update, update_pause_overlay_visibility);
    }
}

#[derive(Component)]
struct PauseOverlay;

fn spawn_pause_overlay(mut commands: Commands) {
    // Create a full-screen overlay positioned over the hourglass
    commands
        .spawn((
            PauseOverlay,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(0.0),
                top: Val::Percent(0.0),
                width: Val::Px(200.0),
                height: Val::Px(100.0),
                display: Display::None, // Start hidden
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)), // Semi-transparent black background
            ZIndex(100), // Ensure it appears above the hourglass
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    // Center the text within the overlay
                    ..default()
                },
            ));
        });
}

fn update_pause_overlay_visibility(
    timer_state: Res<TimerState>,
    mut overlay_query: Query<&mut Node, With<PauseOverlay>>,
    mut last_state: Local<Option<bool>>,
) {
    // Only update if the timer state has changed
    let current_running = timer_state.is_running;
    let current_has_time = timer_state.remaining > 0.0;
    let timer_was_started = timer_state.remaining < timer_state.duration; // Timer was started if remaining time is less than duration

    if last_state.is_none() || last_state.unwrap() != current_running {
        for mut node in overlay_query.iter_mut() {
            // Show overlay only when timer is paused (not running) AND it was previously started AND there's still time remaining
            // Don't show when app first starts (ready state)
            node.display = if !current_running && current_has_time && timer_was_started {
                Display::Flex
            } else {
                Display::None
            };
        }
        *last_state = Some(current_running);
    }
}
