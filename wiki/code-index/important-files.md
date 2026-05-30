<!-- wiki:sources: src/main.rs, src/resources.rs, src/timer.rs, src/hourglass.rs, src/ui/mod.rs, src/ui/color_panel.rs, src/ui/shape_panel.rs, src/ui/timer_panel.rs, src/ui/pause_overlay.rs, Cargo.toml, build_wasm.sh -->

# Important Files

Quick reference to the most important files in the codebase. The entire application is ~9 Rust source files under `src/`, plus build configuration.

| File | Module | Purpose |
|------|--------|---------|
| [[src/main.rs\|main.rs]] | [[modules/app]] | `AppPlugin` — composes Bevy + feature plugins, inits resources, spawns camera. |
| [[src/resources.rs\|resources.rs]] | [[modules/resources]] | `HourglassConfig`, `TimerState`, the `ColorMode`/`HourglassShape`/`ShapeMode` enums, and `COLOR_PALETTE`. The shared state every system reads/writes. |
| [[src/timer.rs\|timer.rs]] | [[modules/timer]] | The countdown logic: `update_timer` system + pure `tick_countdown` helper. |
| [[src/hourglass.rs\|hourglass.rs]] | [[modules/hourglass]] | The largest file. Main hourglass rendering, per-shape mesh configs, morphing interpolation, click/drag handling, recreation-on-change. |
| [[src/ui/mod.rs\|ui/mod.rs]] | [[modules/ui-layout]] | `UIPlugin` + `setup_ui_layout` flexbox scaffold + marker components + `TimerPanelVisible` resource. |
| [[src/ui/color_panel.rs\|color_panel.rs]] | [[modules/color-panel]] | Static color swatches, random-color button, rainbow-cycling button + HSL conversion. |
| [[src/ui/shape_panel.rs\|shape_panel.rs]] | [[modules/shape-panel]] | Mini-hourglass shape selectors, random-shape `?` button, morphing `∞` toggle, hover/scale effects. |
| [[src/ui/timer_panel.rs\|timer_panel.rs]] | [[modules/timer-panel]] | Collapsible panel: +/- duration buttons, Start/Pause/Reset, time display. |
| [[src/ui/pause_overlay.rs\|pause_overlay.rs]] | [[modules/pause-overlay]] | The "PAUSED" overlay shown when a started timer is paused mid-run. |
| [[Cargo.toml\|Cargo.toml]] | — | Dependencies (`bevy` 0.16, `bevy_hourglass` 0.2.2, `rand`), feature flags (`dev`/`dev_native`), native vs WASM build profiles. |
| [[build_wasm.sh\|build_wasm.sh]] | [[features/web-build]] | Builds the WASM bundle and runs `wasm-bindgen`. |
| [[CLAUDE.md\|CLAUDE.md]] | — | Project guidance: build/dev commands, architecture summary. |
| [[TESTING.md\|TESTING.md]] | [[references/test-coverage]] | Manual test plan for the color-change fixes (the parts no unit test covers). |

## External crate of note

`bevy_hourglass` (v0.2.2) provides the `Hourglass` component, `HourglassMeshBuilder`, the `BulbStyle`/`NeckStyle` shape primitives, and `SandSplash` particles. Almost everything visual is delegated to it; this app's job is to *configure and re-configure* it in response to timer state and user input. See [[modules/hourglass]] and [[patterns#Recreate-on-change rendering]].

## Related Pages

- [[code-index/entry-points]]
- [[architecture/overview]]
- [[HOME]]
