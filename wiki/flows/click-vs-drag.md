<!-- wiki:sources: src/hourglass.rs, src/ui/shape_panel.rs -->

# Flow: Click vs. Drag

## Purpose

How a single left-button interaction on the hourglass is resolved into either a **click** (toggle pause/play) or a **drag** (flip + restart), and how presses on overlapping controls are excluded.

Supports: [[features/hourglass-interaction]].

## Entry Points

`handle_hourglass_click` in [[src/hourglass.rs|hourglass.rs]] (`Update`).

## Sequence Diagram

```mermaid
sequenceDiagram
    participant Mouse as Mouse input
    participant HHC as handle_hourglass_click
    participant Guard as Control hit-test
    participant DS as DragState
    participant TS as TimerState
    participant HG as Hourglass

    Mouse->>HHC: cursor + button state
    HHC->>HHC: cursor -> world coords (via Camera2d)
    HHC->>Guard: initial press over main hourglass, not a control?
    Guard-->>HHC: no -> ignore
    Mouse->>DS: just_pressed -> capture gesture, store start_position
    Mouse->>DS: pressed & moved > 10px anywhere in canvas -> is_dragging=true
    Mouse->>HHC: just_released anywhere in canvas
    alt is_dragging
        HHC->>HG: chambers reset, flip()
        HHC->>TS: reset() + is_running=true
    else simple click
        HHC->>TS: toggle is_running
    end
    HHC->>DS: clear active gesture state
```

## Step-by-Step Execution

1. **Cursor → world** — [[src/hourglass.rs|hourglass.rs]]`:handle_hourglass_click`: reads the window cursor and projects it through the `Camera2d` with `viewport_to_world_2d`.
2. **Initial control exclusion** — when the button is first pressed, ignore it if the cursor is over a [[modules/shape-panel|`MiniHourglass`]] sprite (distance < `30 * scale`) or any Bevy UI button (`Interaction != Interaction::None`). This stops shape/color selection from also toggling the timer.
3. **Initial bounds check** — capture the gesture only if the press begins inside the main hourglass hit area (scale-aware in the Chrome extension).
4. **Press capture** (`just_pressed`) — set `is_active`, record `start_position`, and clear `is_dragging`.
5. **Move while pressed** — while the captured gesture is active, keep tracking anywhere in the canvas. If the cursor moves more than `drag_threshold` (10 px), set `is_dragging = true`.
6. **Release** (`just_released`):
   - **Drag** → if `hourglass.can_flip()`: snap chambers to all-sand-in-bottom, call `flip()`, then send `TimerCommand::Restart`.
   - **Click** → send `TimerCommand::Toggle`.
7. **Cleanup** — clear `is_active`, `is_dragging`, and `start_position`. If the browser drops a release while focus changes, cancel the active gesture once the button is no longer pressed.

Separately, [[modules/hourglass#handle_timer_start|`handle_timer_start`]] watches for the not-running→running edge and flips the hourglass **only on the very first start**, so a plain click-to-resume doesn't re-flip — and it skips even that flip when a color/shape change has queued one via `PendingFlip` (see [[modules/hourglass#Flip-on-change orchestration]]).

## Important Files

| File | Role in Flow |
|------|-------------|
| [[src/hourglass.rs\|hourglass.rs]] | `handle_hourglass_click`, `DragState`, `handle_timer_start`. |
| [[src/ui/shape_panel.rs\|shape_panel.rs]] | Defines `MiniHourglass` (the sprites excluded in step 2). |

## Data and State

`DragState` (per-entity component on the main hourglass) captures the in-flight gesture from its valid starting press through release; `TimerState` is the durable outcome. The drag threshold (10 px) is the single tunable separating intents.

## Error Paths

- **No window / camera / hourglass** — the system does nothing that frame.
- **No cursor / projection failure on the initial press** — no gesture starts. A previously captured gesture can still finish on release without another projection.
- **Flip while already flipping** — guarded by `hourglass.can_flip()`.

## Related Pages

- [[features/hourglass-interaction]]
- [[modules/hourglass]], [[modules/shape-panel]]
