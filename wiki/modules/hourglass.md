<!-- wiki:sources: src/hourglass.rs -->

# Hourglass (Rendering & Interaction)

## Responsibility

The visual and interactive heart of the app. This module owns the *main* hourglass entity: it builds it, keeps its sand color/level in sync with state, rebuilds it when shape or color changes, drives the morphing animation, and handles click/drag input on it. It is by far the largest source file ([[src/hourglass.rs|src/hourglass.rs]], ~1000 lines including tests).

## Where It Lives

[[src/hourglass.rs|src/hourglass.rs]]

## Systems (registered by `HourglassPlugin`)

| System | Schedule | Role |
|--------|----------|------|
| `spawn_hourglass` | `Startup` | Build the initial hourglass from default config + timer. |
| `apply_pending_flip` | `Update`, **before** the two rebuild systems | If [[modules/resources#PendingFlip\|`PendingFlip`]] is set, flip the freshly (re)spawned hourglass. |
| `update_hourglass_color` | `Update` | On config change, set `sand_color` and particle color in place. |
| `update_hourglass_shape` | `Update` | On shape/mode/color-mode change (Static shape mode), despawn + rebuild. |
| `update_morphing_shape` | `Update` | In Morphing mode, rebuild each tick with an interpolated shape. |
| `update_hourglass_timer` | `Update`, after `update_morphing_shape` | Copy `TimerState` → `Hourglass` chambers every frame. |
| `handle_hourglass_click` | `Update` | Click = pause/play; drag = flip + reset. |
| `handle_timer_start` | `Update` | Flip the hourglass on the *first* start only — unless a `PendingFlip` already owns the flip. |

## Components

- **`MainHourglass`** (pub) — marks the one real hourglass entity. Used everywhere to disambiguate it from the mini-hourglasses in [[modules/shape-panel]].
- **`DragState`** — per-entity drag tracking: `is_dragging`, `start_position`, and a `drag_threshold` of 10 px that separates a click from a drag.

## Shape presets

`get_main_shape_config(shape)` returns a `(HourglassMeshBodyConfig, HourglassMeshPlatesConfig)` pair for each of the four [[modules/resources#Enums|`HourglassShape`]] variants, at a base height of 400 px:

| Shape | Bulb | Neck | Notes |
|-------|------|------|-------|
| `Classic` | Circular, curvature 1.0 | Curved | The default. |
| `Modern` | Circular, curvature 0.0 (flat) | Straight | Angular look. |
| `Slim` | Circular, width 0.7 | Curved, thin | 1.2× taller, narrower. |
| `Wide` | Circular, width 1.2 | Curved, thick | 0.8× shorter, wider. |

`get_mini_shape_config(shape)` (pub) is the same table at a 25 px base height with lower curve resolutions, used by [[modules/shape-panel]] for the selector buttons.

## The recreate-on-change pattern

`bevy_hourglass` does not expose a way to mutate an existing hourglass's *shape* in place. So shape changes are handled by **despawning the entity and building a new one**. Both `update_hourglass_shape` and `update_morphing_shape` follow the same recipe:

1. Read and preserve the current `flipping` flag and `DragState` from the existing entity.
2. **Bail out if `flipping`** — never interrupt a flip animation mid-flight.
3. Despawn the old entity.
4. Recompute `fill_percent = remaining / duration` so the new hourglass shows the correct sand level.
5. Build a fresh hourglass via `HourglassMeshBuilder`, re-insert `MainHourglass` + preserved `DragState`.
6. `update_hourglass_timer` restores the running/chamber state next frame.

`update_hourglass_shape` only acts in `ShapeMode::Static`. It tracks the last shape, shape mode, and **color mode** in `Local` state and rebuilds when any changed — and *also* rebuilds on static color changes (a documented fix: changing color in place left the sand body stale; see [[references/test-coverage|TESTING.md history]]). Rainbow color changes are throttled to avoid rebuilding the mesh every frame. See [[flows/appearance-recreation]] for the full decision tree. This is the project's signature pattern — [[patterns#Recreate-on-change rendering]].

## Flip-on-change orchestration

A color or shape change doesn't just rebuild the hourglass — it also **flips** it (so the sand visibly resets to the top). But the flip can't be issued at the click site: the rebuild despawns the current entity, so a flip applied there would land on the about-to-die entity and, worse, set its `flipping` flag — which the rebuild guard reads as "don't interrupt a flip," silently dropping the color/shape change entirely. The flip has to land on the *new* entity, one frame later.

The three-step handshake, mediated by the [[modules/resources#PendingFlip|`PendingFlip`]] resource:

1. **Request** — the [[modules/color-panel]] / [[modules/shape-panel]] click handlers set `pending_flip.0 = true` (alongside `timer_state.reset()` + start).
2. **Rebuild** — `update_hourglass_shape` / `update_morphing_shape` despawn the old entity and spawn the replacement that same tick.
3. **Apply** — `apply_pending_flip` runs (ordered `.before(update_hourglass_shape).before(update_morphing_shape)`) and flips the fresh entity via an `Added<MainHourglass>` query, which only matches the new entity the frame *after* the rebuild command flushes. It mirrors the drag-flip: snap `upper_chamber = 0.0` / `lower_chamber = 1.0` so the crate's end-of-flip chamber swap leaves the top full, then `flip()`. The flag is cleared **only once the flip actually fires** (guarded by `can_flip()`), so a request made while a prior flip was still blocking the rebuild survives to the real respawn.

The companion guard lives in `handle_timer_start`: because a color/shape change also starts the timer, the first-start flip would otherwise fire on the *old* entity (tripping the same rebuild-dropping bug). So `handle_timer_start` **skips its flip whenever `PendingFlip` is set** — the queued flip owns the animation. See [[flows/appearance-recreation#Flipping the rebuilt hourglass]].

## Morphing

`update_morphing_shape` runs only in `ShapeMode::Morphing`, throttled to ~every 0.01 s. It computes a cycle position `t = (elapsed % 8.0) / 8.0` (full loop every 8 s) and calls `get_morphed_shape_config(t)`, which:

- Maps `t` onto the 4-shape ring (`Classic → Modern → Slim → Wide → Classic`).
- Interpolates `total_height`, plate width/height, and the bulb/neck styles between the two bracketing shapes.

The interpolation helpers are pure and tested:

- **`lerp_f32(a, b, t)`** — plain linear interpolation, no clamping (extrapolates outside `[0,1]`).
- **`interpolate_bulb_style`** — lerps two `Circular` bulbs (resolution floored at 5); for *mismatched* variants it hard-switches at `t = 0.5`.
- **`interpolate_neck_style`** — lerps `Curved↔Curved` and `Straight↔Straight`; for mixed pairs it converts to `Curved` and ramps curvature from/to 0.

## Input handling

`handle_hourglass_click` (detailed in [[flows/click-vs-drag]]):

- Converts cursor → world coords through the camera.
- **Ignores clicks over controls**: skips if the cursor is over a mini-hourglass sprite button (`MiniHourglass`, radius-based) or any Bevy UI button (`Interaction != None`). Without this, selecting a shape/color would also toggle pause.
- Within ~400 px of the hourglass center: tracks press → release. If movement exceeded the 10 px threshold it's a **drag** → flip + `timer_state.reset()` + start running. Otherwise it's a **click** → toggle `is_running`.

`handle_timer_start` flips the hourglass **only the first time** the timer transitions to running (tracked by a `has_ever_started` `Local`), so resuming from pause doesn't re-flip. The flag resets when the timer is reset to full. It also **defers to a pending flip**: when `PendingFlip` is set (a color/shape change is queuing its own flip on the rebuilt entity), `handle_timer_start` suppresses the first-start flip — see [[modules/hourglass#Flip-on-change orchestration|Flip-on-change orchestration]].

## Features Supported

- [[features/countdown-timer]] — `update_hourglass_timer` mirrors state into the visual.
- [[features/hourglass-interaction]] — click/drag + first-start flip.
- [[features/color-selection]] — `update_hourglass_color` + rebuild path.
- [[features/shape-selection]] — `update_hourglass_shape` + presets.
- [[features/shape-morphing]] — `update_morphing_shape` + interpolation.

## Dependencies

- `bevy_hourglass` — `Hourglass`, `HourglassMeshBuilder`, `BulbStyle`, `NeckStyle`, `SandSplash`, mesh configs.
- `bevy` — meshes, materials, input, camera, transforms.
- [[modules/resources]] — `HourglassConfig`, `TimerState`, `PendingFlip`, the enums.
- [[modules/shape-panel]] — imports `MiniHourglass` to exclude its buttons from clicks.

## Tests

27 tests — the most of any module. Pure logic: `lerp_f32` endpoints/extrapolation, `interpolate_bulb_style` (midpoint, resolution floor, variant switch), `interpolate_neck_style` (curved, straight, both mixed directions), `get_morphed_shape_config` anchors/midpoints/wrap, and the extracted hit-test geometry `within_click_radius` / `exceeds_drag_threshold` (exclusive boundaries, scale, offset center). Headless-`App` system tests: `apply_pending_flip`, `handle_timer_start` (first-start flip, pending-flip suppression, at-rest no-op), `update_hourglass_timer` (state sync + chamber math + zero-duration guard), and `update_hourglass_color`. Still manual-only (camera/window-gated): `spawn_hourglass`, the rebuild systems, and the world-space click dispatch in `handle_hourglass_click` — see [[TESTING.md|TESTING.md]] and [[references/test-coverage#hourglass.rs]].

## Open Questions

- The recreate-on-change approach rebuilds the full mesh on every static color change. Whether this causes a visible hitch on slower web hardware isn't measured here.

## Related Pages

- [[flows/appearance-recreation]], [[flows/click-vs-drag]], [[flows/countdown-tick]]
- [[patterns#Recreate-on-change rendering]]
- [[modules/shape-panel]]
