use crate::resources::TimerState;
use bevy::prelude::*;

pub struct TimerPlugin;

/// The ordered stages used by every system that can affect the timer.
///
/// Input systems emit commands, the timer applies them in one place, the
/// countdown advances, and observers (such as extension persistence) see the
/// resulting state last.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimerSystems {
    Restore,
    Input,
    Apply,
    Deadline,
    Tick,
    Observe,
}

/// Every semantic timer transition goes through this event. Keeping frame-by-
/// frame ticking out of UI systems gives the Chrome extension a clean signal
/// for persistence and alarm scheduling.
#[derive(Event, Debug, Clone, Copy, PartialEq)]
pub enum TimerCommand {
    Start,
    Pause,
    Toggle,
    Reset,
    Restart,
    Adjust(f32),
    Finish,
}

/// Emitted after a command changes timer state, and once when a countdown
/// reaches zero.
#[derive(Event, Debug, Clone, Copy, PartialEq)]
pub struct TimerStateChanged(pub TimerCommand);

impl Plugin for TimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TimerCommand>()
            .add_event::<TimerStateChanged>()
            .configure_sets(
                Update,
                (
                    TimerSystems::Restore,
                    TimerSystems::Input,
                    TimerSystems::Apply,
                    TimerSystems::Deadline,
                    TimerSystems::Tick,
                    TimerSystems::Observe,
                )
                    .chain(),
            )
            .add_systems(Update, apply_timer_commands.in_set(TimerSystems::Apply));

        #[cfg(not(all(feature = "chrome_extension", target_arch = "wasm32")))]
        app.add_systems(Update, update_timer.in_set(TimerSystems::Tick));
    }
}

fn apply_timer_commands(
    mut commands: EventReader<TimerCommand>,
    mut changed: EventWriter<TimerStateChanged>,
    mut timer_state: ResMut<TimerState>,
) {
    for command in commands.read().copied() {
        if apply_timer_command(&mut timer_state, command) {
            changed.write(TimerStateChanged(command));
        }
    }
}

fn apply_timer_command(timer_state: &mut TimerState, command: TimerCommand) -> bool {
    let previous = timer_state.clone();

    match command {
        TimerCommand::Start => timer_state.is_running = true,
        TimerCommand::Pause => timer_state.is_running = false,
        TimerCommand::Toggle => timer_state.is_running = !timer_state.is_running,
        TimerCommand::Reset => timer_state.reset(),
        TimerCommand::Restart => {
            timer_state.reset();
            timer_state.is_running = true;
        }
        TimerCommand::Adjust(seconds) => timer_state.add_time(seconds),
        TimerCommand::Finish => {
            timer_state.remaining = 0.0;
            timer_state.is_running = false;
        }
    }

    // Restart represents a deliberate new run even if the old state happened
    // to be at the same values, so observers must always see it.
    command == TimerCommand::Restart || *timer_state != previous
}

#[cfg(not(all(feature = "chrome_extension", target_arch = "wasm32")))]
fn update_timer(
    time: Res<Time>,
    mut timer_state: ResMut<TimerState>,
    mut changed: EventWriter<TimerStateChanged>,
) {
    if timer_state.is_running && timer_state.remaining > 0.0 {
        let (remaining, is_running) = tick_countdown(timer_state.remaining, time.delta_secs());
        timer_state.remaining = remaining;
        timer_state.is_running = is_running;
        if !is_running {
            changed.write(TimerStateChanged(TimerCommand::Finish));
        }
    }
}

/// Advance the countdown by `delta` seconds, returning the new remaining time
/// and whether the timer is still running. Remaining is clamped to 0 and the
/// timer stops once it reaches 0 (a `delta` larger than `remaining` still
/// yields 0, never a negative value).
#[cfg(any(test, not(all(feature = "chrome_extension", target_arch = "wasm32"))))]
fn tick_countdown(remaining: f32, delta: f32) -> (f32, bool) {
    let new_remaining = remaining - delta;
    if new_remaining <= 0.0 {
        (0.0, false)
    } else {
        (new_remaining, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(duration: f32, remaining: f32, is_running: bool) -> TimerState {
        TimerState {
            duration,
            remaining,
            is_running,
        }
    }

    #[test]
    fn commands_cover_all_semantic_transitions() {
        let mut timer = state(180.0, 90.0, false);
        assert!(apply_timer_command(&mut timer, TimerCommand::Start));
        assert!(timer.is_running);

        assert!(apply_timer_command(&mut timer, TimerCommand::Adjust(30.0)));
        assert_eq!((timer.duration, timer.remaining), (210.0, 120.0));

        assert!(apply_timer_command(&mut timer, TimerCommand::Pause));
        assert!(!timer.is_running);
        assert!(apply_timer_command(&mut timer, TimerCommand::Toggle));
        assert!(timer.is_running);

        assert!(apply_timer_command(&mut timer, TimerCommand::Finish));
        assert_eq!(timer.remaining, 0.0);
        assert!(!timer.is_running);

        assert!(apply_timer_command(&mut timer, TimerCommand::Reset));
        assert_eq!(timer.remaining, 210.0);
        assert!(!timer.is_running);

        assert!(apply_timer_command(&mut timer, TimerCommand::Restart));
        assert_eq!(timer.remaining, 210.0);
        assert!(timer.is_running);
    }

    #[test]
    fn no_op_command_does_not_report_change() {
        let mut timer = state(180.0, 180.0, false);
        assert!(!apply_timer_command(&mut timer, TimerCommand::Pause));
    }

    #[test]
    fn normal_tick_decrements_and_keeps_running() {
        assert_eq!(tick_countdown(10.0, 1.0), (9.0, true));
    }

    #[test]
    fn exact_zero_stops() {
        assert_eq!(tick_countdown(1.0, 1.0), (0.0, false));
    }

    #[test]
    fn overshoot_clamps_to_zero() {
        assert_eq!(tick_countdown(0.5, 2.0), (0.0, false));
    }

    #[test]
    fn zero_delta_leaves_remaining_unchanged() {
        assert_eq!(tick_countdown(10.0, 0.0), (10.0, true));
    }

    #[test]
    fn small_delta_decrements() {
        let (remaining, running) = tick_countdown(10.0, 0.016);
        assert!((remaining - 9.984).abs() < 1e-5);
        assert!(running);
    }
}
