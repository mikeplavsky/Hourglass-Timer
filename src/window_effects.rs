use crate::resources::TimerState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow; // Not in the prelude.

pub struct WindowEffectsPlugin;

impl Plugin for WindowEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowMaximized>()
            .add_systems(Update, maximize_on_completion);
    }
}

/// Tracks whether we drove the OS window into a maximized state when the
/// countdown finished. Serves as both the one-shot guard (maximize only once
/// per completion) and the restore memory (only un-maximize a window that we
/// maximized). Reflects what we *requested*, not the live OS state, so a user
/// un-maximizing via the window button won't be re-maximized. Newtype resource
/// mirroring `PendingFlip` / `TimerPanelVisible`.
#[derive(Resource, Default)]
pub struct WindowMaximized(pub bool);

/// Whether to maximize: the countdown has just stopped at zero and we have not
/// already maximized. A user *pause* leaves `remaining > 0`, so it never trips
/// this; only a natural finish (`remaining <= 0`) does. `duration > 0.0`
/// excludes a degenerate 0/0 timer sitting at rest.
fn should_maximize(is_running: bool, remaining: f32, duration: f32, already: bool) -> bool {
    !already && !is_running && remaining <= 0.0 && duration > 0.0
}

/// Whether to restore: a new countdown has put time back on the clock while we
/// are still in our maximized state. Intentionally independent of `is_running`
/// so that adding time after a finish also restores the window.
fn should_restore(already: bool, remaining: f32) -> bool {
    already && remaining > 0.0
}

/// Maximizes the primary window when the countdown reaches zero and restores it
/// once a new countdown starts. A no-op when there is no single primary window.
fn maximize_on_completion(
    timer_state: Res<TimerState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut maximized: ResMut<WindowMaximized>,
) {
    if let Ok(mut window) = windows.single_mut() {
        if should_maximize(
            timer_state.is_running,
            timer_state.remaining,
            timer_state.duration,
            maximized.0,
        ) {
            window.set_maximized(true);
            maximized.0 = true;
        } else if should_restore(maximized.0, timer_state.remaining) {
            window.set_maximized(false);
            maximized.0 = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- should_maximize --------------------------------------------------

    #[test]
    fn maximizes_on_natural_completion() {
        // Countdown stopped at exactly zero, not yet maximized: the core case.
        assert!(should_maximize(false, 0.0, 180.0, false));
    }

    #[test]
    fn no_maximize_while_running() {
        assert!(!should_maximize(true, 0.0, 180.0, false));
    }

    #[test]
    fn no_maximize_at_rest_before_start() {
        // remaining == duration: never started, nothing to maximize for.
        assert!(!should_maximize(false, 180.0, 180.0, false));
    }

    #[test]
    fn no_maximize_when_already_latched() {
        // The one-shot guard: don't re-fire while we're still finished.
        assert!(!should_maximize(false, 0.0, 180.0, true));
    }

    #[test]
    fn no_maximize_on_user_pause() {
        // A pause leaves time on the clock (remaining > 0) — must not maximize.
        assert!(!should_maximize(false, 50.0, 100.0, false));
    }

    #[test]
    fn no_maximize_for_zero_duration_timer() {
        // Degenerate 0/0 timer at rest: the duration guard suppresses it.
        assert!(!should_maximize(false, 0.0, 0.0, false));
    }

    #[test]
    fn maximizes_with_tiny_overshoot_clamped_to_zero() {
        // tick_countdown clamps remaining to exactly 0.0 on completion.
        assert!(should_maximize(false, 0.0, 0.01, false));
    }

    // --- should_restore ---------------------------------------------------

    #[test]
    fn restores_after_restart() {
        assert!(should_restore(true, 180.0));
    }

    #[test]
    fn no_restore_when_not_latched() {
        // We never maximized, so there's nothing to restore.
        assert!(!should_restore(false, 180.0));
    }

    #[test]
    fn no_restore_while_still_at_zero() {
        assert!(!should_restore(true, 0.0));
    }

    #[test]
    fn restores_with_a_sliver_of_time() {
        assert!(should_restore(true, 0.01));
    }

    // --- maximize_on_completion (headless wiring) -------------------------

    /// App with a `Window` + `PrimaryWindow` spawned in `Startup`, the given
    /// `TimerState`, and `maximize_on_completion` in `Update`. The window is
    /// spawned (not just inserted) so it persists across multiple ticks. One
    /// `update()` runs Startup + a first Update tick.
    ///
    /// No `WinitPlugin` is present (bare `App::new()`), so nothing consumes the
    /// maximize request between the system writing it and a test reading it.
    fn effects_app(timer_state: TimerState) -> App {
        let mut app = App::new();
        app.insert_resource(timer_state);
        app.init_resource::<WindowMaximized>();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((Window::default(), PrimaryWindow));
        });
        app.add_systems(Update, maximize_on_completion);
        app.update();
        app
    }

    /// Consumes and returns the pending maximize request on the primary window.
    /// `take_maximize_request` empties the slot, so a follow-up call sees `None`
    /// unless the system wrote again.
    fn take_request(app: &mut App) -> Option<bool> {
        let mut query = app
            .world_mut()
            .query_filtered::<&mut Window, With<PrimaryWindow>>();
        let world = app.world_mut();
        query
            .single_mut(world)
            .unwrap()
            .internal
            .take_maximize_request()
    }

    /// The non-destructive view of our latch (unlike `take_request`).
    fn is_latched(app: &App) -> bool {
        app.world().resource::<WindowMaximized>().0
    }

    fn set_timer(app: &mut App, remaining: f32, is_running: bool) {
        let mut ts = app.world_mut().resource_mut::<TimerState>();
        ts.remaining = remaining;
        ts.is_running = is_running;
    }

    #[test]
    fn completion_requests_maximize() {
        let mut app = effects_app(TimerState {
            duration: 180.0,
            remaining: 0.0,
            is_running: false,
        });
        assert_eq!(take_request(&mut app), Some(true));
        assert!(is_latched(&app));
    }

    #[test]
    fn at_rest_does_not_request_maximize() {
        // Headline safety: a freshly started app must not maximize on its own.
        let mut app = effects_app(TimerState {
            duration: 180.0,
            remaining: 180.0,
            is_running: false,
        });
        assert_eq!(take_request(&mut app), None);
        assert!(!is_latched(&app));
    }

    #[test]
    fn running_does_not_request_maximize() {
        let mut app = effects_app(TimerState {
            duration: 100.0,
            remaining: 50.0,
            is_running: true,
        });
        assert_eq!(take_request(&mut app), None);
        assert!(!is_latched(&app));
    }

    #[test]
    fn pause_does_not_request_maximize() {
        // Paused mid-run (remaining > 0) — distinct from a finish.
        let mut app = effects_app(TimerState {
            duration: 100.0,
            remaining: 50.0,
            is_running: false,
        });
        assert_eq!(take_request(&mut app), None);
        assert!(!is_latched(&app));
    }

    #[test]
    fn maximize_fires_exactly_once() {
        let mut app = effects_app(TimerState {
            duration: 180.0,
            remaining: 0.0,
            is_running: false,
        });
        assert_eq!(take_request(&mut app), Some(true));
        // Still finished on the next tick: the latch suppresses a second request.
        app.update();
        assert_eq!(take_request(&mut app), None);
        assert!(is_latched(&app));
    }

    #[test]
    fn restart_requests_restore_then_settles() {
        // Full lifecycle in one app: finish -> idle at zero -> restart.
        let mut app = effects_app(TimerState {
            duration: 180.0,
            remaining: 0.0,
            is_running: false,
        });
        assert_eq!(take_request(&mut app), Some(true));
        assert!(is_latched(&app));

        // Sit at zero for a couple of frames: no further requests, still latched.
        app.update();
        assert_eq!(take_request(&mut app), None);
        assert!(is_latched(&app));

        // Restart the countdown (mirrors reset()/drag-flip putting time back).
        set_timer(&mut app, 180.0, true);
        app.update();
        assert_eq!(take_request(&mut app), Some(false));
        assert!(!is_latched(&app));

        // Running on: no thrash.
        set_timer(&mut app, 175.0, true);
        app.update();
        assert_eq!(take_request(&mut app), None);
        assert!(!is_latched(&app));
    }

    #[test]
    fn add_time_after_finish_restores() {
        // Adding time past a finish leaves is_running false but remaining > 0;
        // should_restore ignores is_running, so the window still restores.
        let mut app = effects_app(TimerState {
            duration: 180.0,
            remaining: 0.0,
            is_running: false,
        });
        assert_eq!(take_request(&mut app), Some(true));

        set_timer(&mut app, 60.0, false);
        app.update();
        assert_eq!(take_request(&mut app), Some(false));
        assert!(!is_latched(&app));
    }
}
