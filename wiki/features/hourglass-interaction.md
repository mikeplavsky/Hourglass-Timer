<!-- wiki:sources: src/hourglass.rs, src/ui/pause_overlay.rs, src/resources.rs -->

# Hourglass Interaction

## What It Does

Makes the hourglass itself the primary control. A **click** toggles pause/play; a **drag** flips the hourglass and restarts the timer from full. When a running timer is paused mid-countdown, a **"PAUSED"** overlay appears.

## User Journey

1. User **clicks** the hourglass → if stopped, it starts (and flips on the very first start); if running, it pauses (overlay shows).
2. User **drags** the hourglass (moves the cursor past a 10 px threshold while pressed) → it flips, the timer resets to full, and starts running again.
3. While paused mid-run, the **"PAUSED"** banner is visible; clicking again resumes and hides it.

## Implementation

| Component | File | Role |
|-----------|------|------|
| click/drag | [[src/hourglass.rs\|hourglass.rs]] | `handle_hourglass_click` — distinguishes click vs. drag, toggles or flips+restarts. |
| first-start flip | [[src/hourglass.rs\|hourglass.rs]] | `handle_timer_start` — flips only on the first start, unless a flip is pending. |
| color/shape flip | [[src/hourglass.rs\|hourglass.rs]] | `apply_pending_flip` — flips the rebuilt hourglass when [[modules/resources#PendingFlip\|`PendingFlip`]] is set. |
| drag tracking | [[src/hourglass.rs\|hourglass.rs]] | `DragState { is_active, is_dragging, start_position, drag_threshold: 10.0 }`. |
| pause banner | [[src/ui/pause_overlay.rs\|pause_overlay.rs]] | `update_pause_overlay_visibility`. |

Key modules: [[modules/hourglass]], [[modules/pause-overlay]].

## Click vs. drag

`handle_hourglass_click` converts the cursor to world coordinates and captures a gesture when the initial press lands on the hourglass. The captured gesture continues tracking even when the pointer leaves the scaled hourglass hit area or crosses a control, which is especially important in Chrome's narrow side panel. Moving more than the 10 px `drag_threshold` makes it an overturn; releasing then calls `hourglass.flip()` and sends `TimerCommand::Restart`. A short captured press remains a click and sends `TimerCommand::Toggle`. Full step-by-step in [[flows/click-vs-drag]].

**Crucially**, the handler refuses to start a gesture if the initial press lands on a [[modules/shape-panel|mini-hourglass sprite]] (`MiniHourglass`) or any Bevy UI button (`Interaction != None`). Once a valid hourglass gesture starts, crossing a control no longer interrupts it. Without the initial guard, selecting a shape or color would also toggle the timer — a bug that was explicitly fixed (git history: "Stop shape selection from toggling timer pause").

## First-start flip

`handle_timer_start` flips the hourglass **only on the first** not-running→running transition (tracked by a `has_ever_started` `Local`). Resuming from a pause doesn't re-flip. The flag resets when the timer returns to full.

## Flip on color/shape change

In the Chrome extension, changing the color or shape also flips the hourglass, so the sand visibly resets to the top with the restarted countdown. Because the change rebuilds the entity, the flip is **queued** through [[modules/resources#PendingFlip|`PendingFlip`]] and applied to the new entity on the next frame. Native and ordinary web builds do not request this appearance-change flip. The full extension handshake is in [[modules/hourglass#Flip-on-change orchestration]] and [[flows/appearance-recreation#Flipping the rebuilt hourglass]].

## Pause overlay

`update_pause_overlay_visibility` shows the banner only when the timer is paused **and** has time left **and** had already been started — so it never appears on the fresh app or after completion. See [[modules/pause-overlay]].

## Architecture Decisions

- **Threshold-based click/drag** rather than separate gestures — a single left-button interaction serves both, with 10 px disambiguating intent.
- **Captured press-drag-release** so only the initial press must hit the hourglass; a natural overturning swipe can finish anywhere in the canvas.
- **Hit-test exclusion of controls at gesture start** because the world-space hourglass and the control buttons overlap in screen space.

## Flow

See [[flows/click-vs-drag]].

## Open Questions

- The ~400 px click radius is a fixed approximation of the hourglass bounds, not the actual mesh extent; very wide/slim shapes may be slightly over- or under-covered.
