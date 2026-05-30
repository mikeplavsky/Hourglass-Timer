<!-- wiki:sources: src/hourglass.rs, src/timer.rs, src/ui/color_panel.rs, src/ui/shape_panel.rs, src/ui/timer_panel.rs, src/ui/pause_overlay.rs, build_wasm.sh -->

# Features

The Hourglass Timer is a single-screen visual countdown. At its core is a [[features/countdown-timer|countdown timer]] you start, pause, and reset — either by clicking the hourglass directly ([[features/hourglass-interaction|hourglass interaction]]) or via the collapsible control panel, where you also adjust the duration ([[features/timer-duration-controls|duration controls]]). The timer drives a live [`bevy_hourglass`](https://crates.io/crates/bevy_hourglass) rendering whose sand level mirrors the time remaining.

Around that core sits a layer of **appearance customization**: pick a sand [[features/color-selection|color]] (a fixed swatch, a random color, or a continuously-cycling rainbow), pick one of four hourglass [[features/shape-selection|shapes]] (or a random one), or enable [[features/shape-morphing|morphing]] so the shape smoothly cycles through all four over time. A [[features/hourglass-interaction#Pause overlay|pause overlay]] reads "PAUSED" when a running timer is paused mid-countdown. Finally, the whole app ships to the web as a [[features/web-build|WASM build]].

A cross-cutting behavior worth calling out: **changing the color or shape restarts the countdown** from full and starts it running. This is intentional (see commits and [[features/color-selection#Side effect: restarts the timer]]), and it's why the appearance systems all hold a `ResMut<TimerState>`.

## Feature Inventory

| Feature | Status | Key Module(s) |
|---------|--------|---------------|
| [[features/countdown-timer]] | complete | [[modules/timer]], [[modules/resources]], [[modules/hourglass]] |
| [[features/timer-duration-controls]] | complete | [[modules/timer-panel]], [[modules/resources]] |
| [[features/hourglass-interaction]] | complete | [[modules/hourglass]], [[modules/pause-overlay]] |
| [[features/color-selection]] | complete | [[modules/color-panel]], [[modules/hourglass]] |
| [[features/shape-selection]] | complete | [[modules/shape-panel]], [[modules/hourglass]] |
| [[features/shape-morphing]] | complete | [[modules/hourglass]], [[modules/shape-panel]] |
| [[features/web-build]] | complete | [[Cargo.toml\|Cargo.toml]], [[build_wasm.sh\|build_wasm.sh]] |

## Test coverage at a glance

The pure logic behind these features is unit-tested; the ECS wiring and rendering are verified manually. See [[references/test-coverage]] for the full feature × test matrix.

## Related Pages

- [[architecture/overview]]
- [[HOME]]
- [[references/test-coverage]]
