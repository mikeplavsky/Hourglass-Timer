<!-- wiki:sources: src/ui/shape_panel.rs -->

# Shape Panel

## Responsibility

Implements the shape selector row: four clickable **mini-hourglasses** (one per shape), a `?` random-shape button, and an `∞` morphing toggle. Unlike the [[modules/color-panel]] (which uses Bevy UI nodes), these are **world-space sprites** positioned and hit-tested manually, so they can render actual little hourglass meshes and scale on hover.

## Where It Lives

[[src/ui/shape_panel.rs|src/ui/shape_panel.rs]]

## Systems (registered by `ShapePanelPlugin`)

| System | Schedule | Role |
|--------|----------|------|
| `spawn_shape_buttons` | `PostStartup` | Build 4 mini-hourglasses (Classic/Modern/Slim/Wide). |
| `spawn_random_shape_button` | `PostStartup` | Build the `?` sprite button. |
| `spawn_morphing_button` | `PostStartup` | Build the `∞` sprite button. |
| `handle_shape_button_clicks` | `Update` | Click a mini → set `shape_type`, mode `Static`, restart timer. |
| `handle_random_shape_button_clicks` | `Update` | Click `?` → pick a distinct random shape, restart timer. |
| `handle_morphing_button_clicks` | `Update` | Click `∞` → toggle `ShapeMode::Morphing`. |
| `update_mini_hourglass_colors` | `Update` | Keep mini sand color in sync with config. |
| `handle_hover_effects` | `Update` | Tag the hovered sprite with `HoveredHourglass`. |
| `update_hourglass_layering` | `Update` | Scale sprites: 1.3 hovered, 1.15 selected, 1.0 default. |
| `update_hover_timers` | `Update` | Tick the hover timer. |
| `update_mini_hourglass_positions` | `Update` | Reposition sprites to follow the shape-row UI node. |

## Components

- **`MiniHourglass`** (pub) — marks a shape-row sprite; stores `base_position` and `original_x` (offset from row center). Also imported by [[modules/hourglass]] so the main click handler can *exclude* these buttons.
- **`ShapeButton { shape }`** — tags a mini with which `HourglassShape` it selects.
- **`MorphingButton`**, **`RandomShapeButton`** — tag the `∞` and `?` sprites.
- **`HoveredHourglass { timer }`** — transient tag added to whichever sprite the cursor is over.

## How the sprites are built and positioned

`spawn_shape_buttons` builds each mini via `HourglassMeshBuilder` with `get_mini_shape_config` (25 px presets from [[modules/hourglass#Shape presets]]), filled to 70% for visual appeal, then **removes the `Hourglass` component** so they're static displays, not live timers. They're spawned at a temporary position; `update_mini_hourglass_positions` then anchors them each frame to the world-space projection of the shape-row UI node (center x ± `original_x`, fixed screen y ≈ 60, z = 10). This keeps the world-space sprites visually aligned with the flexbox row across window resizes.

The `?` and `∞` buttons are plain `Mesh2d` rectangles with a child `Text2d` glyph. They use an **embedded font** (`FiraSans-Regular.ttf`, bundled via `embedded_asset!`) because Bevy's default font is an ASCII-only subset that can't render `∞`. See [[patterns#Embedded font for non-ASCII glyphs]].

## Hover & selection feedback

`handle_hover_effects` does distance-based hit testing (radius scaled by current sprite scale) and maintains a single `HoveredHourglass` tag. `update_hourglass_layering` reads that plus the current config to pick a scale: **1.3× hovered, 1.15× selected** (shape matches config, or morphing active for `∞`), else **1.0×**. The `?` button has no persistent selected state — it's a momentary action.

## Side effects: restarts the timer

`handle_shape_button_clicks` and `handle_random_shape_button_clicks` both `reset()` + start the timer — selecting a shape restarts the countdown (git history: "restart timer on shape change"). `handle_morphing_button_clicks` only toggles the mode and does **not** restart. `pick_distinct_shape` re-rolls until it returns a shape different from the current one.

## Features Supported

- [[features/shape-selection]] — the 4 shapes + random.
- [[features/shape-morphing]] — the `∞` toggle (animation itself lives in [[modules/hourglass]]).

## Dependencies

- `bevy` / `bevy::asset::embedded_asset` — sprites, text, embedded font.
- `bevy_hourglass` — mini-hourglass meshes, `HourglassMeshSandState`.
- `rand` — random shape.
- [[modules/resources]], [[modules/ui-layout]] (`ShapeRowMarker`), [[modules/hourglass]] (`get_mini_shape_config`).

## Tests

6 unit tests. `pick_distinct_shape` (always differs from current, returns a valid variant, deterministic per seed) and `shape_button_scale` (hover beats selection → 1.3, selected → 1.15, else 1.0). The hover/click *systems* call `viewport_to_world_2d`, which can't run in a headless `App`, so their hit-test geometry is extracted into `within_click_radius` (tested over in [[modules/hourglass]]) rather than exercised through the system; spawning/positioning remain manual-only. See [[references/test-coverage#shape_panel.rs]].

## Related Pages

- [[features/shape-selection]], [[features/shape-morphing]]
- [[modules/hourglass]]
