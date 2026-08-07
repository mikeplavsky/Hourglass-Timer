<!-- wiki:sources: src/ui/shape_panel.rs, src/hourglass.rs, src/resources.rs -->

# Shape Selection

## What It Does

Lets the user choose the hourglass silhouette from four presets — **Classic**, **Modern**, **Slim**, **Wide** — by clicking a little live preview of each, or roll a **random** shape with the `?` button.

## User Journey

1. User sees four mini-hourglasses in the shape row, each a real (static) preview of a shape.
2. User clicks one → the main hourglass rebuilds in that shape; the selected mini scales up slightly (1.15×).
3. Or clicks the **`?`** button → a random *different* shape is chosen.
4. In the Chrome extension, the countdown **restarts from full, starts running, and the hourglass flips**. Native and ordinary web builds only change the shape.

## Implementation

| Component | File | Role |
|-----------|------|------|
| selector sprites | [[src/ui/shape_panel.rs\|shape_panel.rs]] | Spawns/positions the 4 minis + `?` button; handles clicks, hover scaling. |
| shape presets | [[src/hourglass.rs\|hourglass.rs]] | `get_main_shape_config` / `get_mini_shape_config` define each shape's mesh. |
| state | [[src/resources.rs\|resources.rs]] | `shape_type: HourglassShape`, `shape_mode`. |
| rebuild | [[src/hourglass.rs\|hourglass.rs]] | `update_hourglass_shape` despawns + rebuilds on shape change. |

Key modules: [[modules/shape-panel]], [[modules/hourglass]].

## The four shapes

Defined in `get_main_shape_config` (see the table in [[modules/hourglass#Shape presets]]): Classic (curved, default), Modern (flat bulb + straight neck), Slim (taller, narrower), Wide (shorter, wider). The mini-previews use the same parameters scaled to 25 px (`get_mini_shape_config`) with their `Hourglass` component removed so they're static.

## Selecting a shape

`handle_shape_button_clicks` hit-tests the cursor against each mini (`within_click_radius`, radius `30 * scale`) and, on a hit, sets `config.shape_type` and forces `ShapeMode::Static`. `handle_random_shape_button_clicks` uses `pick_distinct_shape`, which re-rolls until it differs from the current shape. With `chrome_extension` enabled, both also restart the timer and set `PendingFlip`; native and ordinary web builds do not. The visual swap happens in [[modules/hourglass#update_hourglass_shape|`update_hourglass_shape`]], which rebuilds the entity, after which the extension's queued flip lands via `apply_pending_flip`.

## Visual feedback

`update_hourglass_layering` scales each mini: **1.3×** when hovered, **1.15×** when its shape is the selected one, **1.0×** otherwise. The `?` button only scales on hover (it's a momentary action with no selected state).

## Architecture Decisions

- **Live mini-previews** (real meshes, not icons) so the selector shows exactly what you'll get. They're world-space sprites, not UI nodes — see [[patterns#Dual UI: nodes vs. world sprites]].
- **Distinct-shape re-roll** so random always changes the shape.

## Flow

See [[flows/appearance-recreation]] for the rebuild path shared with color changes.

## Open Questions

None notable.
