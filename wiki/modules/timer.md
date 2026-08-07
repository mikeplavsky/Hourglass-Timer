<!-- wiki:sources: src/timer.rs -->

# Timer (Countdown Logic)

## Responsibility

Advances the countdown each frame. This module is small and focused: one Bevy system (`update_timer`) and one pure helper (`tick_countdown`) that contains the actual arithmetic. Splitting the arithmetic out is what makes the countdown unit-testable without spinning up a Bevy `App`.

## Where It Lives

[[src/timer.rs|src/timer.rs]]

## Public Interfaces

`TimerPlugin` registers a single `Update` system, `update_timer`. There are no exported types — the module communicates entirely through the shared [[modules/resources#TimerState|`TimerState`]] resource.

## How It Works

```rust
fn update_timer(time: Res<Time>, mut timer_state: ResMut<TimerState>) {
    if timer_state.is_running && timer_state.remaining > 0.0 {
        let (remaining, is_running) =
            tick_countdown(timer_state.remaining, time.delta_secs());
        timer_state.remaining = remaining;
        timer_state.is_running = is_running;
    }
}
```

The system runs every frame but only does work when the timer `is_running` and has time left. It delegates to:

```rust
fn tick_countdown(remaining: f32, delta: f32) -> (f32, bool) {
    let new_remaining = remaining - delta;
    if new_remaining <= 0.0 { (0.0, false) } else { (new_remaining, true) }
}
```

Key guarantees (all tested):

- **Clamps to zero, never negative** — an overshoot (`delta` > `remaining`) yields `(0.0, false)`, not a negative remaining. This is why [[modules/resources#format_time]]'s negative-value quirk never reaches the screen.
- **Stops at zero** — reaching exactly 0 sets `is_running = false`, so the timer halts on its own.
- **Frame-rate independent** — uses `time.delta_secs()`, so the countdown tracks wall-clock time regardless of FPS.

## Relationship to the hourglass

`update_timer` only mutates `TimerState`. It does **not** touch the visual `Hourglass` component. The bridge is [[modules/hourglass#update_hourglass_timer|`update_hourglass_timer`]], a separate system in [[modules/hourglass]] that copies `TimerState` into the `Hourglass` each frame (chamber fill = `remaining / duration`). This separation keeps the countdown logic pure and the rendering concerns in one place. See [[flows/countdown-tick]].

## Features Supported

- [[features/countdown-timer]] — this *is* the countdown.

## Dependencies

- `bevy` — `Time`, `Res`, `ResMut`.
- [[modules/resources]] — `TimerState`.

## Used By

The hourglass and UI read the `TimerState` this module advances: [[modules/hourglass]], [[modules/timer-panel]], [[modules/pause-overlay]].

## Tests

Inline `#[cfg(test)]` exhaustively covers `tick_countdown`: normal decrement, exact-zero stop, overshoot clamp, zero-delta no-op, small-delta precision. This is the best-covered logic in the project. See [[references/test-coverage#timer.rs]].

## Related Pages

- [[flows/countdown-tick]]
- [[features/countdown-timer]]
- [[modules/resources]]
