use crate::resources::TimerState;
use bevy::prelude::*;

pub struct TimerPlugin;

impl Plugin for TimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_timer);
    }
}

fn update_timer(time: Res<Time>, mut timer_state: ResMut<TimerState>) {
    if timer_state.is_running && timer_state.remaining > 0.0 {
        let (remaining, is_running) = tick_countdown(timer_state.remaining, time.delta_secs());
        timer_state.remaining = remaining;
        timer_state.is_running = is_running;
    }
}

/// Advance the countdown by `delta` seconds, returning the new remaining time
/// and whether the timer is still running. Remaining is clamped to 0 and the
/// timer stops once it reaches 0 (a `delta` larger than `remaining` still
/// yields 0, never a negative value).
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
