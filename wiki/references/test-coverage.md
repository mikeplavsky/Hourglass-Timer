<!-- wiki:sources: src/resources.rs, src/timer.rs, src/hourglass.rs, src/ui/color_panel.rs, src/ui/shape_panel.rs, src/ui/timer_panel.rs, src/ui/pause_overlay.rs, TESTING.md -->

# Test Coverage

This page answers: **how well do the tests cover each feature?** Short version — the project has **43 unit tests** that cover the *pure logic* (timer arithmetic, morph interpolation, color/shape math, distinct-pick re-rolls) very thoroughly, but the **Bevy ECS systems** (spawning, input, rendering-sync, the timer-restart side effects) have **no automated tests** and are verified by hand per [[TESTING.md|TESTING.md]].

The reason for the split is structural and deliberate: testable arithmetic was extracted into free functions (`tick_countdown`, `lerp_f32`, `pick_distinct_color`, …) that run without a Bevy `App`, while everything requiring a live `World`/window stays in systems. See [[patterns#Pure helpers extracted from systems]].

## How to run the tests

This is the recommended way for this project (see [[CLAUDE.local.md|CLAUDE.local.md]] and project memory):

```bash
cargo test --no-default-features
```

`--no-default-features` avoids the `dev_native` feature (dynamic linking / hot-reload) which isn't needed for the logic tests. To run one module's tests, e.g. `cargo test --no-default-features hourglass`.

## Test count by file

| File | Tests | What they cover |
|------|------:|-----------------|
| [[src/hourglass.rs\|hourglass.rs]] | 14 | `lerp_f32`, bulb/neck interpolation, `get_morphed_shape_config` anchors. |
| [[src/ui/color_panel.rs\|color_panel.rs]] | 12 | `color_dist_sq`, `pick_distinct_color`, `rainbow_hue`, `hsl_to_rgb`. |
| [[src/resources.rs\|resources.rs]] | 9 | `reset`, `add_time` clamps, `format_time`. |
| [[src/timer.rs\|timer.rs]] | 5 | `tick_countdown` decrement/clamp/stop. |
| [[src/ui/shape_panel.rs\|shape_panel.rs]] | 3 | `pick_distinct_shape`. |
| [[src/ui/timer_panel.rs\|timer_panel.rs]] | 0 | — (logic lives in `resources.rs`). |
| [[src/ui/pause_overlay.rs\|pause_overlay.rs]] | 0 | — (visibility condition not extracted). |
| [[src/ui/mod.rs\|ui/mod.rs]], [[src/main.rs\|main.rs]] | 0 | — (declarative layout / composition). |

## Feature × Test coverage matrix

**Legend:** 🟢 logic well covered by unit tests · 🟡 partial (core math tested, wiring/side-effects not) · 🔴 no automated tests (manual only).

| Feature | Coverage | Tested (unit) | Not tested (automated) |
|---------|:--------:|---------------|------------------------|
| [[features/countdown-timer]] | 🟢 | `tick_countdown` (all edges), `format_time` | `update_timer` / `update_hourglass_timer` wiring |
| [[features/timer-duration-controls]] | 🟡 | `add_time` clamps, `reset`, `format_time` | button systems, panel show/hide |
| [[features/shape-morphing]] | 🟡 | `lerp_f32`, bulb/neck interpolation, morph anchors | `update_morphing_shape` rebuild, throttle |
| [[features/color-selection]] | 🟡 | distinct-color re-roll, `rainbow_hue`, `hsl_to_rgb` | apply-to-sand systems, **timer-restart side effect** |
| [[features/shape-selection]] | 🟡 | `pick_distinct_shape` | click hit-testing, rebuild, **timer-restart side effect** |
| [[features/hourglass-interaction]] | 🔴 | — | click/drag, first-start flip, control-exclusion guard |
| [[features/web-build]] | 🔴 | — | build script (verified by building) |

## Notable behaviors pinned by tests

A few tests exist specifically to **document quirks**, not just to check happy paths:

- `format_time_negative_is_not_zero_padded` — pins that `format_time(-5)` yields `"00:00:-5"`; safe only because [[modules/timer|`tick_countdown`]] clamps upstream.
- `add_time_clamps_duration_before_remaining` — pins the clamp *ordering* in `add_time`.
- `interpolate_bulb_mixed_variants_switch_at_half` — pins the hard-switch-at-0.5 behavior for mismatched bulb variants.
- `lerp_extrapolates_outside_unit_interval` — pins that `lerp_f32` does **not** clamp.

## Biggest coverage gaps (where bugs could hide untested)

1. **Timer-restart side effects** — that picking a color/shape resets and starts the timer is core behavior with no automated test (only the underlying `reset()` is tested). It lives in the click handlers in [[modules/color-panel]] / [[modules/shape-panel]].
2. **Control-exclusion guard** in [[modules/hourglass#Input handling|`handle_hourglass_click`]] — the logic that stops shape/color clicks from toggling pause. Regressing it wouldn't fail any test.
3. **Pause overlay condition** — the three-part visibility rule in [[modules/pause-overlay]] is inline in the system; extracting it into a pure helper would make it testable.
4. **Recreate-on-change preservation** — that drag/flip state survives a rebuild ([[flows/appearance-recreation]]) is untested.

## Manual test plan

[[TESTING.md|TESTING.md]] documents the hand-testing steps for the color-change fixes (static/random/rainbow application to both sand body and particles, and mode transitions) — exactly the system-level behavior the unit tests don't reach.

## Related Pages

- [[features/overview]]
- [[patterns#Pure helpers extracted from systems]]
- the per-module `Tests` sections, e.g. [[modules/timer#Tests]], [[modules/hourglass#Tests]]
