<!-- wiki:sources: src/hourglass.rs, src/ui/color_panel.rs, src/ui/shape_panel.rs, src/resources.rs -->

# Flow: Appearance Recreation

## Purpose

How a color or shape change becomes a visibly-updated hourglass. This is the project's signature mechanism: because `bevy_hourglass` builds its mesh from config and can't restyle in place, most appearance changes **despawn the hourglass and build a new one**, carefully preserving timer/drag state.

Supports: [[features/color-selection]], [[features/shape-selection]], [[features/shape-morphing]].

## Entry Points

- `update_hourglass_color` — in-place color (sand + particles).
- `update_hourglass_shape` — rebuild on shape/mode/static-color change (Static shape mode).
- `update_morphing_shape` — rebuild each tick (Morphing mode).

All in [[src/hourglass.rs|hourglass.rs]] (`Update`).

## Sequence Diagram

```mermaid
sequenceDiagram
    participant UI as Color/Shape panel
    participant Cfg as HourglassConfig
    participant UHS as update_hourglass_shape
    participant Old as Old Hourglass
    participant New as New Hourglass
    participant UHT as update_hourglass_timer

    UI->>Cfg: set color / shape / mode (is_changed)
    Cfg->>UHS: detect change (Local last_* trackers)
    UHS->>Old: read flipping + DragState
    alt currently flipping
        UHS-->>UHS: return (don't interrupt)
    else
        UHS->>UHS: fill_percent = remaining/duration
        UHS->>Old: despawn
        UHS->>New: HourglassMeshBuilder.build(...)
        UHS->>New: insert MainHourglass + preserved DragState
        UHT->>New: next frame: restore running + chambers
    end
```

## Decision logic in `update_hourglass_shape`

Active only in `ShapeMode::Static`. It keeps four `Local` trackers (`last_shape_type`, `last_shape_mode`, `last_color_mode`, `last_recreation_time`) and decides:

| Condition | Action |
|-----------|--------|
| shape, shape-mode, or color-mode changed | rebuild immediately |
| config changed, `ColorMode::Rainbow` | rebuild, but **throttled** (~0.01 s) to keep particles visible |
| config changed, `ColorMode::Static` | rebuild (so the sand *body* updates, not just particles) |
| nothing changed | return early |

Then, before rebuilding: read the old entity's `flipping` + `DragState`; **if flipping, return** (never interrupt a flip); else despawn, compute `fill_percent = remaining/duration`, build via `HourglassMeshBuilder` (body + plates + sand + sand-splash + timing), and re-insert `MainHourglass` + the preserved `DragState`. State is restored next frame by `update_hourglass_timer`.

`update_morphing_shape` follows the identical preserve→guard→despawn→rebuild recipe, but is driven by time (`t = elapsed % 8 / 8`) rather than a config change, and feeds an interpolated config from `get_morphed_shape_config`.

## Why two systems touch color

`update_hourglass_color` updates `sand_color` and particle color **in place** every change — cheap, but it does *not* refresh the sand body mesh. The static-color rebuild in `update_hourglass_shape` exists precisely to fix that gap (documented in [[TESTING.md|TESTING.md]]: "only sand particles would change color but the main sand body would remain the old color"). So a static color change triggers both: the in-place update *and* a rebuild.

## Important Files

| File | Role in Flow |
|------|-------------|
| [[src/hourglass.rs\|hourglass.rs]] | The three update systems + `HourglassMeshBuilder` calls. |
| [[src/ui/color_panel.rs\|color_panel.rs]], [[src/ui/shape_panel.rs\|shape_panel.rs]] | Write the config that triggers recreation. |

## Data and State

The hourglass entity is transient — it may be destroyed and rebuilt many times per second (morphing). Durable state lives in `TimerState` + `HourglassConfig` (resources) and the preserved `DragState`; the entity is just a view of them.

## Error Paths

- **Mid-flip change** — deferred (system returns) until the flip animation completes.
- **`duration == 0`** — `fill_percent` falls back to `1.0`.

## Related Pages

- [[patterns#Recreate-on-change rendering]]
- [[modules/hourglass]]
- [[features/color-selection]], [[features/shape-selection]], [[features/shape-morphing]]
