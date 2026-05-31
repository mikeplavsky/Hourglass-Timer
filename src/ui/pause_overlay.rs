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
    // Only update if the running state has changed
    let current_running = timer_state.is_running;

    if last_state.is_none() || last_state.unwrap() != current_running {
        for mut node in overlay_query.iter_mut() {
            // Show overlay only when timer is paused (not running) AND it was previously started AND there's still time remaining
            // Don't show when app first starts (ready state)
            node.display = if pause_overlay_should_show(
                current_running,
                timer_state.remaining,
                timer_state.duration,
            ) {
                Display::Flex
            } else {
                Display::None
            };
        }
        *last_state = Some(current_running);
    }
}

/// Whether the "PAUSED" overlay should be visible: only when the timer is
/// paused (not running), was previously started (`remaining < duration`), and
/// still has time left (`remaining > 0`). At rest before the first start, and
/// once finished, the overlay stays hidden.
fn pause_overlay_should_show(is_running: bool, remaining: f32, duration: f32) -> bool {
    !is_running && remaining > 0.0 && remaining < duration
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- pause_overlay_should_show ----------------------------------------

    #[test]
    fn shows_when_paused_mid_run() {
        assert!(pause_overlay_should_show(false, 50.0, 100.0));
    }

    #[test]
    fn hidden_while_running() {
        assert!(!pause_overlay_should_show(true, 50.0, 100.0));
    }

    #[test]
    fn hidden_when_finished() {
        // remaining == 0: countdown done, no overlay.
        assert!(!pause_overlay_should_show(false, 0.0, 100.0));
    }

    #[test]
    fn hidden_before_first_start() {
        // remaining == duration: never started, so nothing to resume.
        assert!(!pause_overlay_should_show(false, 100.0, 100.0));
    }

    #[test]
    fn shows_with_a_sliver_of_time_left() {
        assert!(pause_overlay_should_show(false, 0.01, 100.0));
    }

    // --- update_pause_overlay_visibility (headless wiring) ----------------

    /// One-tick app: a `PauseOverlay` node (initially hidden) and the given
    /// `TimerState`, with `update_pause_overlay_visibility` in `Update`. The
    /// `Local<Option<bool>>` starts `None`, so the body runs on the first tick.
    fn overlay_app(timer_state: TimerState) -> App {
        let mut app = App::new();
        app.insert_resource(timer_state);
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                PauseOverlay,
                Node {
                    display: Display::None,
                    ..default()
                },
            ));
        });
        app.add_systems(Update, update_pause_overlay_visibility);
        app.update();
        app
    }

    fn overlay_display(app: &mut App) -> Display {
        let mut query = app
            .world_mut()
            .query_filtered::<&Node, With<PauseOverlay>>();
        query.single(app.world()).unwrap().display
    }

    #[test]
    fn paused_mid_run_makes_overlay_flex() {
        let mut app = overlay_app(TimerState {
            duration: 100.0,
            remaining: 50.0,
            is_running: false,
        });
        assert_eq!(overlay_display(&mut app), Display::Flex);
    }

    #[test]
    fn running_keeps_overlay_hidden() {
        let mut app = overlay_app(TimerState {
            duration: 100.0,
            remaining: 50.0,
            is_running: true,
        });
        assert_eq!(overlay_display(&mut app), Display::None);
    }
}
