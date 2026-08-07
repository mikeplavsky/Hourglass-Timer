<!-- wiki:sources: src/main.rs, src/hourglass.rs, src/ui/mod.rs, src/ui/color_panel.rs, src/ui/shape_panel.rs, src/ui/timer_panel.rs, src/ui/pause_overlay.rs -->

# Entry Points

Where execution begins in this codebase. As a [Bevy](https://bevyengine.org/) app, there is one process `main()` that assembles plugins; from there control is driven by Bevy's ECS schedule. The "entry points" that matter most for understanding behavior are therefore the **system registrations** inside each plugin's `build()` — these are the functions Bevy calls every frame (`Update`) or once (`Startup`/`PostStartup`).

| Entry Point | File | Type | Description |
|-------------|------|------|-------------|
| `main()` | [[src/main.rs\|main.rs]] | Process main | Builds `App`, adds `AppPlugin`, calls `run()`. Returns `AppExit`. |
| `AppPlugin::build` | [[src/main.rs\|main.rs]] | Plugin root | Adds `DefaultPlugins` (window titled "Hourglass Timer"), inits [[modules/resources\|resources]], adds the three feature plugins, spawns the 2D camera. |
| `spawn_camera` | [[src/main.rs\|main.rs]] | `Startup` system | Spawns the single `Camera2d` used for all world→screen projection. |
| `spawn_hourglass` | [[src/hourglass.rs\|hourglass.rs]] | `Startup` system | Builds the main hourglass entity from the default [[modules/resources#HourglassConfig\|config]] and [[modules/resources#TimerState\|timer state]]. |
| `setup_ui_layout` | [[src/ui/mod.rs\|ui/mod.rs]] | `Startup` system | Spawns the root flexbox UI tree with marker nodes the panels attach to. |
| `spawn_color_buttons` | [[src/ui/color_panel.rs\|color_panel.rs]] | `PostStartup` system | Attaches color swatches + random/rainbow buttons under the color row. |
| `spawn_shape_buttons` / `spawn_random_shape_button` / `spawn_morphing_button` | [[src/ui/shape_panel.rs\|shape_panel.rs]] | `PostStartup` systems | Spawn the mini-hourglass shape selectors and the `?` / `∞` sprite buttons. |
| `spawn_timer_controls` | [[src/ui/timer_panel.rs\|timer_panel.rs]] | `PostStartup` system | Spawns the collapsible timer-controls panel + toggle button. |
| `spawn_pause_overlay` | [[src/ui/pause_overlay.rs\|pause_overlay.rs]] | `Startup` system | Spawns the hidden "PAUSED" overlay. |

## Why `PostStartup` for the panels

The UI panels (`color_panel`, `shape_panel`, `timer_panel`) spawn their buttons in **`PostStartup`**, not `Startup`. This is deliberate: `setup_ui_layout` in [[src/ui/mod.rs\|ui/mod.rs]] runs during `Startup` and creates the marker container nodes (`ColorRowMarker`, `ShapeRowMarker`, `BottomTimerMarker`). The panel spawners query for those markers and attach children to them — so they must run *after* the layout exists. `PostStartup` guarantees that ordering without explicit system ordering constraints.

## Related Pages

- [[architecture/overview]] — how the plugins compose
- [[flows/startup]] — the full startup sequence as a diagram
- [[code-index/important-files]]
