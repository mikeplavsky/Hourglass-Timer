<!-- wiki:sources: src/timer.rs, src/hourglass.rs, src/resources.rs, src/ui/mod.rs, src/ui/color_panel.rs, src/ui/shape_panel.rs -->

# Patterns and Conventions

The recurring design patterns in this codebase. Most are idiomatic Bevy ECS; a couple are specific workarounds for how `bevy_hourglass` behaves.

## Resource-mediated communication

**What**: Plugins never reference each other. They share state only through two global `Resource`s — `HourglassConfig` and `TimerState` — and react to changes via `is_changed()` / `Changed<>` filters.
**Where**: [[modules/resources]] defines them; every other module reads/writes them.
**Why**: Keeps plugins independently understandable and composable. The UI can change the color without knowing the hourglass exists; the hourglass reacts to config changes without knowing which button caused them.

A consequence worth internalizing: to trace any behavior, follow the *resource*, not a call graph. "What restarts the timer?" → grep for `ResMut<TimerState>`. See [[flows/countdown-tick]].

Used by: every feature.

## Pure helpers extracted from systems

**What**: Arithmetic-heavy logic is pulled out of Bevy systems into free functions that take plain values and return plain values — so they run in a unit test without a Bevy `App`, window, or `World`.
**Where**: `tick_countdown` ([[src/timer.rs|timer.rs]]); `lerp_f32`/`interpolate_bulb_style`/`interpolate_neck_style`/`get_morphed_shape_config` ([[src/hourglass.rs|hourglass.rs]]); `pick_distinct_color`/`color_dist_sq`/`rainbow_hue`/`hsl_to_rgb` ([[src/ui/color_panel.rs|color_panel.rs]]); `pick_distinct_shape` ([[src/ui/shape_panel.rs|shape_panel.rs]]); `add_time`/`format_time`/`reset` ([[src/resources.rs|resources.rs]]).
**Why**: Testability. This single discipline is why 43 tests exist for an app whose rendering is otherwise hard to test. The flip side — anything left *inside* a system is untested. See [[references/test-coverage]].

Used by: [[features/countdown-timer]], [[features/shape-morphing]], [[features/color-selection]], [[features/shape-selection]].

## Recreate-on-change rendering

**What**: To change the hourglass's shape (or fully refresh its color), the entity is **despawned and rebuilt** from config via `HourglassMeshBuilder`, rather than mutated in place. Transient gesture/animation state (`DragState`, `flipping`) is read off the old entity and re-applied; timer state is restored next frame by `update_hourglass_timer`.
**Where**: [[modules/hourglass]] — `update_hourglass_shape`, `update_morphing_shape`.
**Why**: `bevy_hourglass` derives its mesh from a config struct and doesn't expose in-place re-styling. Two guards make it safe: skip while `flipping`, and throttle high-frequency rebuilds (rainbow, morphing) to ~0.01 s. See [[flows/appearance-recreation]].

Used by: [[features/shape-selection]], [[features/shape-morphing]], [[features/color-selection]].

## Dual UI: nodes vs. world sprites

**What**: Two parallel UI systems coexist. The color row and timer panel are **Bevy UI nodes** (flexbox layout, `Interaction`-based clicks). The shape selectors are **world-space sprites** (`Mesh2d` / mini-hourglasses) positioned in world coordinates and hit-tested by distance.
**Where**: nodes in [[modules/ui-layout]], [[modules/color-panel]], [[modules/timer-panel]]; sprites in [[modules/shape-panel]].
**Why**: The shape selectors *are* real little hourglasses (the best possible preview), which is natural as world meshes but awkward as UI nodes. The trade-off: `handle_hourglass_click` must explicitly exclude both kinds of control from counting as hourglass clicks (see [[flows/click-vs-drag]]), and `update_mini_hourglass_positions` must manually keep the sprites aligned to the UI row.

Used by: [[features/shape-selection]], [[features/hourglass-interaction]].

## `Local` state for change detection

**What**: Systems remember the previous value of something across frames using `Local<T>` parameters, then act only on transitions.
**Where**: `update_hourglass_shape`'s `last_shape_type`/`last_shape_mode`/`last_color_mode`/`last_recreation_time`; `handle_timer_start`'s `last_running_state`/`has_ever_started`; `update_pause_overlay_visibility`'s `last_state`.
**Why**: Lets a per-frame `Update` system fire work only on meaningful edges (shape actually changed, timer *just* started, running flag flipped) without a separate event system.

Used by: [[features/shape-selection]], [[features/hourglass-interaction]], [[features/shape-morphing]].

## Distinct-pick re-roll

**What**: Random pickers loop until the result differs enough from the current value — color by squared-RGB distance ≥ `0.3²`, shape by inequality.
**Where**: `pick_distinct_color` ([[modules/color-panel]]), `pick_distinct_shape` ([[modules/shape-panel]]).
**Why**: A "random" button that sometimes returns the current value feels broken. Re-rolling guarantees visible change.

Used by: [[features/color-selection]], [[features/shape-selection]].

## Embedded font for non-ASCII glyphs

**What**: `FiraSans-Regular.ttf` is compiled into the binary with `embedded_asset!` and loaded via an `embedded://` path.
**Where**: [[modules/shape-panel]] (the `∞` morph button and `?` random button).
**Why**: Bevy's built-in default font is an ASCII-only FiraMono subset that can't render `∞`. Embedding avoids shipping a sibling `assets/` directory at runtime (important for the single-file WASM deploy).

Used by: [[features/shape-morphing]], [[features/shape-selection]].
