use bevy::prelude::*;
use crate::resources::TimerState;

pub struct TimerPlugin;

impl Plugin for TimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_timer);
    }
}

fn update_timer(
    time: Res<Time>,
    mut timer_state: ResMut<TimerState>,
) {
    if timer_state.is_running && timer_state.remaining > 0.0 {
        timer_state.remaining -= time.delta_secs();

        // Stop at 0
        if timer_state.remaining <= 0.0 {
            timer_state.remaining = 0.0;
            timer_state.is_running = false;
        }
    }
}
