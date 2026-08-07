<!-- wiki:sources: src/ui/color_panel.rs -->

# Color Panel

## Responsibility

Implements the color row at the top of the screen: 8 fixed swatches, a random-color button, and a rainbow-cycling button. Writes the chosen color and `ColorMode` into [[modules/resources#HourglassConfig|`HourglassConfig`]]; the [[modules/hourglass]] systems react to the change.

## Where It Lives

[[src/ui/color_panel.rs|src/ui/color_panel.rs]]

## Systems (registered by `ColorPanelPlugin`)

| System | Schedule | Role |
|--------|----------|------|
| `spawn_color_buttons` | `PostStartup` | Build swatches + random + rainbow buttons under `ColorRowMarker`. |
| `handle_color_button_clicks` | `Update` | Static swatch → set color and mode `Static`; extension also restarts and requests a flip. |
| `handle_random_color_button` | `Update` | Pick a distinct random color and mode `Random`; extension also restarts and requests a flip. |
| `handle_rainbow_color_button` | `Update` | Set mode `Rainbow`; extension also restarts and requests a flip. |
| `update_rainbow_color` | `Update` | While in `Rainbow`, advance the hue each frame. |

## The three color modes

- **Static** — `handle_color_button_clicks` reads the `ColorButton.color` from one of the `COLOR_PALETTE` swatches and assigns it. Border turns green on press.
- **Random** — `handle_random_color_button` rolls a random RGB color via `pick_distinct_color`, which **re-rolls until the new color is far enough from the current one** (squared RGB distance ≥ `0.3²`). This guarantees a visible change rather than an imperceptible one.
- **Rainbow** — the button just sets the mode; `update_rainbow_color` then continuously sets `config.color = hsl_to_rgb(rainbow_hue(elapsed), 1.0, 0.5)` every frame, cycling hue 0→360° once per 6 s.

## Pure helpers (tested)

| Helper | Purpose |
|--------|---------|
| `color_dist_sq(a, b)` | Squared Euclidean RGB distance (no `sqrt` — only compared to a threshold). |
| `pick_distinct_color(current, min_dist_sq, rng)` | Random color guaranteed `≥ min_dist_sq` from `current`. |
| `rainbow_hue(elapsed_secs)` | `(elapsed * 60) % 360` — hue for the rainbow animation. |
| `hsl_to_rgb(h, s, l)` | Manual HSL→RGB conversion for vibrant rainbow colors. |

## Side effects: restart the timer and flip

On the Chrome extension target, the three click handlers restart the countdown from full, start it, and set `pending_flip.0 = true`. The flip itself can't happen inline because the color change rebuilds the hourglass entity, so the request is handed to [[modules/hourglass#Flip-on-change orchestration|`apply_pending_flip`]]. Native and ordinary web builds only change the color/mode. `update_rainbow_color`'s per-frame hue updates never restart or flip; only the initial extension button press does.

## Features Supported

- [[features/color-selection]] — this module is its primary implementation.

## Dependencies

- `bevy` — UI nodes, `Button`, `Interaction`, `Color`.
- `rand` — random color generation.
- [[modules/resources]] — `HourglassConfig`, `ColorMode`, `COLOR_PALETTE`, `TimerState`, `PendingFlip`.
- [[modules/ui-layout]] — `ColorRowMarker`.

## Used By

Reacts are consumed by [[modules/hourglass]] (`update_hourglass_color`, the rebuild path) and [[modules/shape-panel]] (`update_mini_hourglass_colors`).

## Tests

Well-covered pure logic: `color_dist_sq` (identity, black↔white = 3.0, symmetry, threshold), `pick_distinct_color` (respects min distance across seeds, channels in range, deterministic at zero threshold), `rainbow_hue` (cycle points, in-range sweep), `hsl_to_rgb` (primary/secondary hues, zero-saturation gray, lightness extremes). The systems and the timer-restart side effect are untested. See [[references/test-coverage#color_panel.rs]].

## Related Pages

- [[features/color-selection]]
- [[modules/hourglass]]
