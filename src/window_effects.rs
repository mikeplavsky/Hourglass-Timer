use crate::resources::TimerState;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowMode}; // Not in the prelude (MonitorSelection is).

pub struct WindowEffectsPlugin;

impl Plugin for WindowEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowFullscreen>()
            .add_systems(Update, fullscreen_on_completion);
    }
}

/// The fullscreen mode we switch to on completion: borderless windowed
/// fullscreen on the window's current monitor. Instant and reversible, with no
/// display-mode/resolution change (unlike exclusive `WindowMode::Fullscreen`).
const FULLSCREEN: WindowMode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);

/// Tracks whether we drove the OS window into fullscreen when the countdown
/// finished. Serves as both the one-shot guard (enter fullscreen only once per
/// completion) and the restore memory (only return to windowed a window that we
/// sent fullscreen). Reflects what we *requested*, not the live OS state, so a
/// user toggling fullscreen themselves won't be overridden. Newtype resource
/// mirroring `PendingFlip` / `TimerPanelVisible`.
#[derive(Resource, Default)]
pub struct WindowFullscreen(pub bool);

/// Whether to go fullscreen: the countdown has just stopped at zero and we are
/// not already fullscreen. A user *pause* leaves `remaining > 0`, so it never
/// trips this; only a natural finish (`remaining <= 0`) does. `duration > 0.0`
/// excludes a degenerate 0/0 timer sitting at rest.
fn should_enter_fullscreen(is_running: bool, remaining: f32, duration: f32, already: bool) -> bool {
    !already && !is_running && remaining <= 0.0 && duration > 0.0
}

/// Whether to return to windowed: a new countdown has put time back on the
/// clock while we are still fullscreen. Intentionally independent of
/// `is_running` so that adding time after a finish also restores the window.
fn should_exit_fullscreen(already: bool, remaining: f32) -> bool {
    already && remaining > 0.0
}

/// Sends the primary window fullscreen when the countdown reaches zero and
/// returns it to windowed once a new countdown starts. A no-op when there is no
/// single primary window.
fn fullscreen_on_completion(
    timer_state: Res<TimerState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut fullscreen: ResMut<WindowFullscreen>,
) {
    if let Ok(mut window) = windows.single_mut() {
        if should_enter_fullscreen(
            timer_state.is_running,
            timer_state.remaining,
            timer_state.duration,
            fullscreen.0,
        ) {
            window.mode = FULLSCREEN;
            fullscreen.0 = true;
        } else if should_exit_fullscreen(fullscreen.0, timer_state.remaining) {
            window.mode = WindowMode::Windowed;
            fullscreen.0 = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- should_enter_fullscreen ------------------------------------------

    #[test]
    fn enters_on_natural_completion() {
        // Countdown stopped at exactly zero, not yet fullscreen: the core case.
        assert!(should_enter_fullscreen(false, 0.0, 180.0, false));
    }

    #[test]
    fn no_enter_while_running() {
        assert!(!should_enter_fullscreen(true, 0.0, 180.0, false));
    }

    #[test]
    fn no_enter_at_rest_before_start() {
        // remaining == duration: never started, nothing to go fullscreen for.
        assert!(!should_enter_fullscreen(false, 180.0, 180.0, false));
    }

    #[test]
    fn no_enter_when_already_fullscreen() {
        // The one-shot guard: don't re-fire while we're still finished.
        assert!(!should_enter_fullscreen(false, 0.0, 180.0, true));
    }

    #[test]
    fn no_enter_on_user_pause() {
        // A pause leaves time on the clock (remaining > 0) — must not go fullscreen.
        assert!(!should_enter_fullscreen(false, 50.0, 100.0, false));
    }

    #[test]
    fn no_enter_for_zero_duration_timer() {
        // Degenerate 0/0 timer at rest: the duration guard suppresses it.
        assert!(!should_enter_fullscreen(false, 0.0, 0.0, false));
    }

    #[test]
    fn enters_with_tiny_overshoot_clamped_to_zero() {
        // tick_countdown clamps remaining to exactly 0.0 on completion.
        assert!(should_enter_fullscreen(false, 0.0, 0.01, false));
    }

    // --- should_exit_fullscreen -------------------------------------------

    #[test]
    fn exits_after_restart() {
        assert!(should_exit_fullscreen(true, 180.0));
    }

    #[test]
    fn no_exit_when_not_fullscreen() {
        // We never went fullscreen, so there's nothing to restore.
        assert!(!should_exit_fullscreen(false, 180.0));
    }

    #[test]
    fn no_exit_while_still_at_zero() {
        assert!(!should_exit_fullscreen(true, 0.0));
    }

    #[test]
    fn exits_with_a_sliver_of_time() {
        assert!(should_exit_fullscreen(true, 0.01));
    }

    // --- fullscreen_on_completion (headless wiring) -----------------------

    /// App with a `Window` + `PrimaryWindow` spawned in `Startup`, the given
    /// `TimerState`, and `fullscreen_on_completion` in `Update`. The window is
    /// spawned (not just inserted) so it persists across multiple ticks. One
    /// `update()` runs Startup + a first Update tick.
    ///
    /// No `WinitPlugin` is present (bare `App::new()`), so nothing resets
    /// `window.mode` between the system writing it and a test reading it.
    fn effects_app(timer_state: TimerState) -> App {
        let mut app = App::new();
        app.insert_resource(timer_state);
        app.init_resource::<WindowFullscreen>();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((Window::default(), PrimaryWindow));
        });
        app.add_systems(Update, fullscreen_on_completion);
        app.update();
        app
    }

    /// The primary window's current display mode (`Copy`, read non-destructively).
    fn current_mode(app: &mut App) -> WindowMode {
        let mut query = app
            .world_mut()
            .query_filtered::<&Window, With<PrimaryWindow>>();
        query.single(app.world()).unwrap().mode
    }

    /// Our latch: whether we believe the window is in our fullscreen state.
    fn is_latched(app: &App) -> bool {
        app.world().resource::<WindowFullscreen>().0
    }

    fn set_timer(app: &mut App, remaining: f32, is_running: bool) {
        let mut ts = app.world_mut().resource_mut::<TimerState>();
        ts.remaining = remaining;
        ts.is_running = is_running;
    }

    #[test]
    fn completion_enters_fullscreen() {
        let mut app = effects_app(TimerState {
            duration: 180.0,
            remaining: 0.0,
            is_running: false,
        });
        assert_eq!(current_mode(&mut app), FULLSCREEN);
        assert!(is_latched(&app));
    }

    #[test]
    fn at_rest_stays_windowed() {
        // Headline safety: a freshly started app must not go fullscreen on its own.
        let mut app = effects_app(TimerState {
            duration: 180.0,
            remaining: 180.0,
            is_running: false,
        });
        assert_eq!(current_mode(&mut app), WindowMode::Windowed);
        assert!(!is_latched(&app));
    }

    #[test]
    fn running_stays_windowed() {
        let mut app = effects_app(TimerState {
            duration: 100.0,
            remaining: 50.0,
            is_running: true,
        });
        assert_eq!(current_mode(&mut app), WindowMode::Windowed);
        assert!(!is_latched(&app));
    }

    #[test]
    fn pause_stays_windowed() {
        // Paused mid-run (remaining > 0) — distinct from a finish.
        let mut app = effects_app(TimerState {
            duration: 100.0,
            remaining: 50.0,
            is_running: false,
        });
        assert_eq!(current_mode(&mut app), WindowMode::Windowed);
        assert!(!is_latched(&app));
    }

    #[test]
    fn stays_fullscreen_while_finished() {
        let mut app = effects_app(TimerState {
            duration: 180.0,
            remaining: 0.0,
            is_running: false,
        });
        assert_eq!(current_mode(&mut app), FULLSCREEN);
        // Still finished on the next tick: the latch holds, no thrash back/forth.
        app.update();
        assert_eq!(current_mode(&mut app), FULLSCREEN);
        assert!(is_latched(&app));
    }

    #[test]
    fn restart_returns_to_windowed_then_settles() {
        // Full lifecycle in one app: finish -> idle at zero -> restart.
        let mut app = effects_app(TimerState {
            duration: 180.0,
            remaining: 0.0,
            is_running: false,
        });
        assert_eq!(current_mode(&mut app), FULLSCREEN);
        assert!(is_latched(&app));

        // Sit at zero for a frame: still fullscreen, still latched.
        app.update();
        assert_eq!(current_mode(&mut app), FULLSCREEN);
        assert!(is_latched(&app));

        // Restart the countdown (mirrors reset()/drag-flip putting time back).
        set_timer(&mut app, 180.0, true);
        app.update();
        assert_eq!(current_mode(&mut app), WindowMode::Windowed);
        assert!(!is_latched(&app));

        // Running on: stays windowed, no thrash.
        set_timer(&mut app, 175.0, true);
        app.update();
        assert_eq!(current_mode(&mut app), WindowMode::Windowed);
        assert!(!is_latched(&app));
    }

    #[test]
    fn add_time_after_finish_returns_to_windowed() {
        // Adding time past a finish leaves is_running false but remaining > 0;
        // should_exit_fullscreen ignores is_running, so the window still restores.
        let mut app = effects_app(TimerState {
            duration: 180.0,
            remaining: 0.0,
            is_running: false,
        });
        assert_eq!(current_mode(&mut app), FULLSCREEN);

        set_timer(&mut app, 60.0, false);
        app.update();
        assert_eq!(current_mode(&mut app), WindowMode::Windowed);
        assert!(!is_latched(&app));
    }
}
