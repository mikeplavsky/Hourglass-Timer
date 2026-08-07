<!-- wiki:sources: src/resources.rs, src/timer.rs, src/hourglass.rs, src/chrome_extension.rs, src/ui/color_panel.rs, src/ui/shape_panel.rs, src/ui/timer_panel.rs, src/ui/pause_overlay.rs, extension/tests/state.test.mjs, extension/tests/service-worker.test.mjs, TESTING.md -->

# Test Coverage

This page answers: **how well do the tests cover each feature?** The current tree has **83 regular Rust tests**, **98 Rust tests with `chrome_extension` enabled**, and **13 JavaScript extension tests**. They combine pure-logic unit tests, headless Bevy `App` integration tests, and dependency-free Node tests for snapshot/alarm/lifecycle behavior. Camera/window-gated input and rendered layout still require manual verification per [[TESTING.md|TESTING.md]].

Two complementary patterns make this possible. First, arithmetic-heavy logic is extracted into free functions (`tick_countdown`, `lerp_f32`, `within_click_radius`, `pause_overlay_should_show`, …) that run without a Bevy `App` — see [[patterns#Pure helpers extracted from systems]]. Second, resource-driven systems (timer sync, button handlers, panel/overlay visibility, the first-start flip) are tested in a bare `App::new()`: spawn the entity in `Startup`, run one `app.update()`, then assert on the world. Systems that call `camera.viewport_to_world_2d(...)` can't be driven this way (the projection is unpopulated headless, so the body never runs), so their math is extracted and unit-tested instead.

## How to run the tests

This is the recommended way for this project (see [[CLAUDE.local.md|CLAUDE.local.md]] and project memory):

```bash
cargo test --no-default-features
cargo test --no-default-features --features chrome_extension
node --test extension/tests/*.test.mjs
```

`--no-default-features` avoids the `dev_native` feature (dynamic linking / hot-reload) which isn't needed for the tests. To run one module's tests, e.g. `cargo test --no-default-features hourglass`.

To measure coverage (uses [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov)):

```bash
cargo llvm-cov --no-default-features --html && open target/llvm-cov/html/index.html
```

## Test count by file

| File | Regular / extension | What they cover |
|------|:-------------------:|-----------------|
| [[src/chrome_extension.rs\|chrome_extension.rs]] | 0 / 6 | Snapshot wire format, version rejection, zero-duration restoration, absolute deadlines, expiry, and restart reconciliation. |
| [[src/hourglass.rs\|hourglass.rs]] | 27 / 29 | Pure morph/hit-test helpers; headless flip, timer, color, and extension responsive-scale behavior. |
| [[src/ui/color_panel.rs\|color_panel.rs]] | 12 / 12 | `color_dist_sq`, `pick_distinct_color`, `rainbow_hue`, `hsl_to_rgb`. |
| [[src/ui/timer_panel.rs\|timer_panel.rs]] | 12 / 13 | Time adjustment, playback controls, visibility, display updates, and extension collapsed controls. |
| [[src/resources.rs\|resources.rs]] | 10 / 10 | Default sand state, reset, duration clamps, and formatting. |
| [[src/ui/pause_overlay.rs\|pause_overlay.rs]] | 7 / 8 | Pause visibility plus the extension's text-free overlay. |
| [[src/ui/shape_panel.rs\|shape_panel.rs]] | 7 / 9 | Distinct shapes, button scale, fixed mini-preview sand color, extension layout anchoring, and physical-to-logical coordinate conversion. |
| [[src/timer.rs\|timer.rs]] | 7 / 7 | Semantic timer commands and countdown edges. |
| [[src/ui/mod.rs\|ui/mod.rs]] | 1 / 4 | Extension-only restart/flip gating and sidebar container/layout properties. |
| `extension/tests/*.test.mjs` | 13 JS | Snapshot validation/deadlines/revisions, notification deduplication, live-panel lifecycle, and terminal clearing. |

## Feature × Test coverage matrix

**Legend:** 🟢 logic well covered (unit + headless) · 🟡 partial (core math + some systems tested, camera/window input not) · 🔴 no automated tests (manual only).

| Feature | Coverage | Tested | Not tested (automated) |
|---------|:--------:|--------|------------------------|
| [[features/countdown-timer]] | 🟢 | `tick_countdown` (all edges), `format_time`; `update_hourglass_timer` sync + chamber math (headless) | `update_timer` `Res<Time>` wrapper |
| [[features/timer-duration-controls]] | 🟢 | `add_time` clamps, `reset`, `format_time`; all timer-panel button + visibility + display systems (headless) | — |
| [[features/shape-morphing]] | 🟡 | `lerp_f32`, bulb/neck interpolation, morph anchors | `update_morphing_shape` rebuild, throttle |
| [[features/color-selection]] | 🟡 | distinct-color re-roll, `rainbow_hue`, `hsl_to_rgb`; extension-only restart/flip gating | camera-gated click handlers, splash-particle sync |
| [[features/shape-selection]] | 🟡 | `pick_distinct_shape`, `shape_button_scale`, `within_click_radius`; extension-only restart/flip gating | camera-gated click handlers + rebuild |
| Chrome extension state/lifecycle | 🟢 | zero-duration snapshots, deadlines, stale updates, last-panel clearing, revisions, notifications | real Chrome alarm timing after device sleep |
| [[features/hourglass-interaction]] | 🟡 | first-start flip / pending guard (`handle_timer_start`), `apply_pending_flip`, `within_click_radius`, `exceeds_drag_threshold` | world-space click/drag dispatch (`handle_hourglass_click`), control-exclusion guard |
| [[features/web-build]] | 🔴 | — | build script (verified by building) |

## Notable behaviors pinned by tests

A few tests exist specifically to **document quirks**, not just to check happy paths:

- `format_time_negative_is_not_zero_padded` — pins that `format_time(-5)` yields `"00:00:-5"`; safe only because [[modules/timer|`tick_countdown`]] clamps upstream.
- `add_time_clamps_duration_before_remaining` — pins the clamp *ordering* in `add_time`.
- `interpolate_bulb_mixed_variants_switch_at_half` — pins the hard-switch-at-0.5 behavior for mismatched bulb variants.
- `lerp_extrapolates_outside_unit_interval` — pins that `lerp_f32` does **not** clamp.
- `within_click_radius_boundary_is_exclusive` / `exceeds_drag_threshold_boundary_is_exclusive` — pin the strict `<` / `>` boundary (a click exactly on the radius is a *miss*; a move exactly at the threshold is still a *click*).
- `first_start_skips_flip_when_pending` — pins that a queued color/shape flip suppresses the first-start flip, so the flip lands on the rebuilt entity via `apply_pending_flip` (see [[flows/appearance-recreation]]).
- `update_hourglass_timer_zero_duration_leaves_chambers_default` — pins that a zero duration skips the chamber math (no divide-by-zero) rather than writing `NaN`.

## Biggest coverage gaps (where bugs could hide untested)

1. **Camera-gated click dispatch** — the world-space hit-testing *inside* the click handlers (`handle_hourglass_click`, the shape/morphing/random handlers) can't run in a headless `App` because `viewport_to_world_2d` needs a populated camera projection. The geometry is extracted and tested (`within_click_radius`, `exceeds_drag_threshold`), but the surrounding dispatch — including the **control-exclusion guard** that stops shape/color clicks from toggling pause — is verified only by hand.
2. **Color/shape click dispatch** — extension-only restart/flip gating is unit-tested, but the camera-gated shape handlers and rendered Bevy buttons still need manual interaction testing.
3. **Appearance recreation** — that drag/flip state survives the despawn-and-rebuild ([[flows/appearance-recreation]]), plus the rebuild throttle in `update_morphing_shape` / `update_hourglass_shape`, is untested.
4. **`update_timer` wrapper** — the `Res<Time>` system in [[modules/timer]] is a thin delegate to the (fully tested) `tick_countdown`; the wrapper itself has no test.

## Manual test plan

[[TESTING.md|TESTING.md]] documents the hand-testing steps for the color-change fixes (static/random/rainbow application to both sand body and particles, and mode transitions) — exactly the system-level behavior the unit tests don't reach.

## Related Pages

- [[features/overview]]
- [[patterns#Pure helpers extracted from systems]]
- the per-module `Tests` sections, e.g. [[modules/timer#Tests]], [[modules/hourglass#Tests]]
