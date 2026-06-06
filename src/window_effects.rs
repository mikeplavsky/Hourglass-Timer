use crate::resources::TimerState;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowMode, WindowResized}; // Not in the prelude (MonitorSelection is).

pub struct WindowEffectsPlugin;

impl Plugin for WindowEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowFullscreen>()
            .init_resource::<FullscreenTransition>()
            .add_systems(Startup, spawn_fullscreen_blackout)
            .add_systems(Update, fullscreen_on_completion);
    }
}

/// The fullscreen mode we switch to on completion: borderless windowed
/// fullscreen on the window's current monitor. Instant and reversible, with no
/// display-mode/resolution change (unlike exclusive `WindowMode::Fullscreen`).
const FULLSCREEN: WindowMode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);

/// Maximum number of update ticks to keep the blackout up if the platform does
/// not emit a resize event for the fullscreen transition.
const BLACKOUT_FALLBACK_FRAMES: u32 = 12;

/// Tracks whether we drove the OS window into fullscreen when the countdown
/// finished. Serves as both the one-shot guard (enter fullscreen only once per
/// completion) and the restore memory (only return to windowed a window that we
/// sent fullscreen). Reflects what we *requested*, not the live OS state, so a
/// user toggling fullscreen themselves won't be overridden. Newtype resource
/// mirroring `PendingFlip` / `TimerPanelVisible`.
#[derive(Resource, Default)]
pub struct WindowFullscreen(pub bool);

#[derive(Component)]
struct FullscreenBlackout;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum BlackoutPhase {
    #[default]
    Hidden,
    WaitingForResize,
    SettlingAfterResize,
}

/// Tracks the temporary blackout used to hide OS/window resize artifacts while
/// the app enters fullscreen.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
struct FullscreenTransition {
    phase: BlackoutPhase,
    fallback_frames: u32,
}

impl FullscreenTransition {
    fn start_blackout(&mut self) {
        self.phase = BlackoutPhase::WaitingForResize;
        self.fallback_frames = 0;
    }

    fn clear_blackout(&mut self) {
        self.phase = BlackoutPhase::Hidden;
        self.fallback_frames = 0;
    }

    fn blackout_visible(&self) -> bool {
        self.phase != BlackoutPhase::Hidden
    }

    fn advance_blackout(&mut self, primary_window_resized: bool) {
        match self.phase {
            BlackoutPhase::Hidden => {}
            BlackoutPhase::WaitingForResize if primary_window_resized => {
                self.phase = BlackoutPhase::SettlingAfterResize;
                self.fallback_frames = 0;
            }
            BlackoutPhase::WaitingForResize => {
                self.fallback_frames += 1;
                if self.fallback_frames >= BLACKOUT_FALLBACK_FRAMES {
                    self.clear_blackout();
                }
            }
            BlackoutPhase::SettlingAfterResize => {
                self.clear_blackout();
            }
        }
    }
}

fn spawn_fullscreen_blackout(mut commands: Commands) {
    commands.spawn((
        Name::new("Fullscreen Transition Blackout"),
        FullscreenBlackout,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::BLACK),
        ZIndex(1000),
    ));
}

fn set_blackout_display(
    blackout_visible: bool,
    blackout_query: &mut Query<&mut Node, With<FullscreenBlackout>>,
) {
    let display = if blackout_visible {
        Display::Flex
    } else {
        Display::None
    };

    for mut node in blackout_query.iter_mut() {
        node.display = display;
    }
}

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
    mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    mut fullscreen: ResMut<WindowFullscreen>,
    mut transition: ResMut<FullscreenTransition>,
    mut resize_events: EventReader<WindowResized>,
    mut blackout_query: Query<&mut Node, With<FullscreenBlackout>>,
) {
    if let Ok((window_entity, mut window)) = windows.single_mut() {
        let primary_window_resized = resize_events
            .read()
            .any(|event| event.window == window_entity);

        if should_enter_fullscreen(
            timer_state.is_running,
            timer_state.remaining,
            timer_state.duration,
            fullscreen.0,
        ) {
            transition.start_blackout();
            set_blackout_display(true, &mut blackout_query);
            window.mode = FULLSCREEN;
            fullscreen.0 = true;
        } else if should_exit_fullscreen(fullscreen.0, timer_state.remaining) {
            window.mode = WindowMode::Windowed;
            fullscreen.0 = false;
            transition.clear_blackout();
            set_blackout_display(false, &mut blackout_query);
        } else {
            transition.advance_blackout(primary_window_resized);
            set_blackout_display(transition.blackout_visible(), &mut blackout_query);
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

    /// App with a `Window` + `PrimaryWindow` and blackout overlay spawned in
    /// `Startup`, the given `TimerState`, and `fullscreen_on_completion` in
    /// `Update`. The window is spawned (not just inserted) so it persists
    /// across multiple ticks. One `update()` runs Startup + a first Update tick.
    ///
    /// No `WinitPlugin` is present (bare `App::new()`), so nothing resets
    /// `window.mode` between the system writing it and a test reading it.
    fn effects_app(timer_state: TimerState) -> App {
        let mut app = App::new();
        app.insert_resource(timer_state);
        app.add_event::<WindowResized>();
        app.init_resource::<WindowFullscreen>();
        app.init_resource::<FullscreenTransition>();
        app.add_systems(Startup, (spawn_primary_window, spawn_fullscreen_blackout));
        app.add_systems(Update, fullscreen_on_completion);
        app.update();
        app
    }

    fn spawn_primary_window(mut commands: Commands) {
        commands.spawn((Window::default(), PrimaryWindow));
    }

    /// The primary window's current display mode (`Copy`, read non-destructively).
    fn current_mode(app: &mut App) -> WindowMode {
        let mut query = app
            .world_mut()
            .query_filtered::<&Window, With<PrimaryWindow>>();
        query.single(app.world()).unwrap().mode
    }

    fn primary_window_entity(app: &mut App) -> Entity {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<PrimaryWindow>>();
        query.single(app.world()).unwrap()
    }

    fn send_primary_resize(app: &mut App) {
        let window = primary_window_entity(app);
        app.world_mut().send_event(WindowResized {
            window,
            width: 1920.0,
            height: 1080.0,
        });
    }

    /// Our latch: whether we believe the window is in our fullscreen state.
    fn is_latched(app: &App) -> bool {
        app.world().resource::<WindowFullscreen>().0
    }

    fn blackout_display(app: &mut App) -> Display {
        let mut query = app
            .world_mut()
            .query_filtered::<&Node, With<FullscreenBlackout>>();
        query.single(app.world()).unwrap().display
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
        assert_eq!(blackout_display(&mut app), Display::Flex);
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
        assert_eq!(blackout_display(&mut app), Display::None);
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
        assert_eq!(blackout_display(&mut app), Display::Flex);
    }

    #[test]
    fn blackout_hides_after_resize_plus_settle_frame() {
        let mut app = effects_app(TimerState {
            duration: 180.0,
            remaining: 0.0,
            is_running: false,
        });
        assert_eq!(blackout_display(&mut app), Display::Flex);

        send_primary_resize(&mut app);
        app.update();
        assert_eq!(blackout_display(&mut app), Display::Flex);

        app.update();
        assert_eq!(blackout_display(&mut app), Display::None);
    }

    #[test]
    fn blackout_fallback_prevents_permanent_black_screen() {
        let mut app = effects_app(TimerState {
            duration: 180.0,
            remaining: 0.0,
            is_running: false,
        });
        assert_eq!(blackout_display(&mut app), Display::Flex);

        for _ in 0..BLACKOUT_FALLBACK_FRAMES {
            app.update();
        }

        assert_eq!(blackout_display(&mut app), Display::None);
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
        assert_eq!(blackout_display(&mut app), Display::None);

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
        assert_eq!(blackout_display(&mut app), Display::None);
    }
}
