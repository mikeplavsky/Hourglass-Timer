<!-- wiki:sources: src/resources.rs -->

# Resources (Shared State)

## Responsibility

Defines the two Bevy `Resource`s that hold all application state — `HourglassConfig` (appearance) and `TimerState` (countdown) — plus the enums and color palette they reference. This is the shared vocabulary: nearly every system in the app reads or writes one of these resources, which is how the decoupled plugins communicate without knowing about each other.

## Where It Lives

[[src/resources.rs|src/resources.rs]]

## Public Interfaces

### HourglassConfig

```rust
pub struct HourglassConfig {
    pub color: Color,
    pub shape_type: HourglassShape,
    pub color_mode: ColorMode,
    pub shape_mode: ShapeMode,
}
```

The appearance state. `Default` is a sandy color `srgb(0.8, 0.6, 0.2)`, `Classic` shape, `Static` color mode, `Static` shape mode. Written by the [[modules/color-panel|color panel]] and [[modules/shape-panel|shape panel]]; read by [[modules/hourglass|hourglass]] systems (which react to `is_changed()`).

### TimerState

```rust
pub struct TimerState {
    pub duration: f32,   // total, seconds
    pub remaining: f32,  // remaining, seconds
    pub is_running: bool,
}
```

`Default` is 180 s (3 min), not running. Three methods carry real logic and are unit-tested:

- **`reset()`** — sets `remaining = duration` and `is_running = false`.
- **`add_time(seconds)`** — adds to *both* `duration` and `remaining`, then clamps: `duration` into `0.0..=86400.0` (24 h), then `remaining` into `0.0..=duration`. Order matters — duration is clamped first, so `remaining`'s upper bound uses the already-clamped duration. Negative arguments shrink the timer (the `-1s … -1h` buttons).
- **`format_time()`** — renders `remaining` as `HH:MM:SS` via integer truncation (`as i32`).

### PendingFlip

```rust
pub struct PendingFlip(pub bool);
```

A one-bit signal resource (`Default` is `false`). Chrome-extension color/shape handlers set it to `true` to **request a flip on the next (re)built hourglass**; [[modules/hourglass#Flip-on-change orchestration|`apply_pending_flip`]] consumes it. Native and ordinary web appearance changes leave it false. It exists because a color/shape change rebuilds the entity, so the extension flip must wait for the fresh entity.

### Enums

| Enum | Variants | Meaning |
|------|----------|---------|
| `ColorMode` | `Static`, `Random`, `Rainbow` | How sand color is chosen. `Rainbow` is animated each frame; the others are set once on click. |
| `HourglassShape` | `Classic`, `Modern`, `Slim`, `Wide` | The four mesh presets (see [[modules/hourglass#Shape presets]]). |
| `ShapeMode` | `Static`, `Morphing` | Whether the shape is fixed or continuously interpolating. |

### COLOR_PALETTE

A `&[Color]` of 8 fixed swatches (black, white, blue, red, purple, green, yellow, orange) used to populate the [[modules/color-panel|color row]].

## Features Supported

- [[features/countdown-timer]] — `TimerState` is its entire data model.
- [[features/timer-duration-controls]] — `add_time` / `reset`.
- [[features/color-selection]] — `color`, `color_mode`, `COLOR_PALETTE`; `PendingFlip` (color change flips the hourglass).
- [[features/shape-selection]] / [[features/shape-morphing]] — `shape_type`, `shape_mode`; `PendingFlip` (shape change flips the hourglass).
- [[features/hourglass-interaction]] — `PendingFlip` couples a color/shape change to a flip animation.

## Dependencies

- `bevy` — `Resource`, `Color`.

## Used By

Essentially everything: [[modules/timer]], [[modules/hourglass]], [[modules/color-panel]], [[modules/shape-panel]], [[modules/timer-panel]], [[modules/pause-overlay]].

## Notable Behaviors / Quirks

- **`format_time` does not guard against negative `remaining`** — `state(0.0, -5.0).format_time()` yields `"00:00:-5"`. In practice the countdown clamps `remaining` to 0 upstream (see [[modules/timer]]), so the UI never shows it. This is pinned by a deliberate test (`format_time_negative_is_not_zero_padded`).
- **`add_time` clamp order** is itself a tested behavior (`add_time_clamps_duration_before_remaining`).

## Tests

Inline `#[cfg(test)]` module covers `reset`, `add_time` (positive, negative-clamp, both clamp bounds, normal range), and `format_time` (boundaries, truncation, the negative quirk). Strong coverage of the pure logic. Full list in [[references/test-coverage#resources.rs]].

## Related Pages

- [[modules/timer]] — consumes `TimerState`
- [[architecture/overview]]
