<!-- wiki:sources: src/ui/pause_overlay.rs -->

# Pause Overlay

## Responsibility

Shows a "PAUSED" banner over the hourglass when a *running, mid-countdown* timer is paused — but deliberately not when the app first loads (the ready state) or after the timer finishes. A small but careful piece of state-driven UI.

## Where It Lives

[[src/ui/pause_overlay.rs|src/ui/pause_overlay.rs]]

## Systems (registered by `PauseOverlayPlugin`)

| System | Schedule | Role |
|--------|----------|------|
| `spawn_pause_overlay` | `Startup` | Spawn the hidden overlay node + "PAUSED" text. |
| `update_pause_overlay_visibility` | `Update` | Show/hide based on timer state. |

## The visibility condition

`spawn_pause_overlay` creates an absolutely-positioned 200×100 node, semi-transparent black, `ZIndex(100)` so it sits above the hourglass, starting `Display::None`.

`update_pause_overlay_visibility` delegates the decision to the pure helper `pause_overlay_should_show(is_running, remaining, duration)`, which returns `true` only when **all three** hold:

```rust
fn pause_overlay_should_show(is_running: bool, remaining: f32, duration: f32) -> bool {
    !is_running && remaining > 0.0 && remaining < duration
}
```

- `!is_running` — the timer is paused.
- `remaining > 0.0` — don't show after it hits zero (finished).
- `remaining < duration` — it had been counting down (don't show in the fresh, never-started state).

Extracting the condition is what makes it unit-testable without a Bevy `App` (see [[patterns#Pure helpers extracted from systems]]); the system itself only maps the `bool` to `Display::Flex` / `Display::None`.

The system guards its work behind a `Local<Option<bool>>` (`last_state`) tracking the previous `is_running`, so it only recomputes when the running flag actually changes. (Note: because the gate is keyed on `is_running` alone, the recompute fires on the running→paused / paused→running edges — exactly when the overlay needs to appear or disappear.)

## Features Supported

- [[features/hourglass-interaction]] — specifically the [[features/hourglass-interaction#Pause overlay|pause feedback]].

## Dependencies

- `bevy` — UI node, text, `ZIndex`.
- [[modules/resources]] — `TimerState` (read-only).

## Used By

Bevy runtime. Read-only consumer of `TimerState`.

## Tests

7 tests. The three-way visibility condition is extracted into the pure helper `pause_overlay_should_show(is_running, remaining, duration)`, unit-tested across paused/running/finished/not-started/sliver cases. Two headless-`App` tests then drive `update_pause_overlay_visibility` itself, asserting the node flips to `Display::Flex` when paused mid-run and stays `None` while running. See [[references/test-coverage#pause_overlay.rs]].

## Open Questions

- The overlay node is fixed at the top-left 200×100 region, not centered over the hourglass. Whether this is intentional placement or a known rough edge isn't documented in code.

## Related Pages

- [[features/hourglass-interaction]]
- [[modules/timer]]
