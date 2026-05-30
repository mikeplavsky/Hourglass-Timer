<!-- wiki:sources: src/hourglass.rs, src/ui/pause_overlay.rs -->

# Hourglass Interaction

## What It Does

Makes the hourglass itself the primary control. A **click** toggles pause/play; a **drag** flips the hourglass and resets the timer to full. When a running timer is paused mid-countdown, a **"PAUSED"** overlay appears.

## User Journey

1. User **clicks** the hourglass → if stopped, it starts (and flips on the very first start); if running, it pauses (overlay shows).
2. User **drags** the hourglass (moves the cursor past a 10 px threshold while pressed) → it flips, the timer resets to full, and starts running again.
3. While paused mid-run, the **"PAUSED"** banner is visible; clicking again resumes and hides it.

## Implementation

| Component | File | Role |
|-----------|------|------|
| click/drag | [[src/hourglass.rs\|hourglass.rs]] | `handle_hourglass_click` — distinguishes click vs. drag, toggles or flips+resets. |
| first-start flip | [[src/hourglass.rs\|hourglass.rs]] | `handle_timer_start` — flips only on the first start. |
| drag tracking | [[src/hourglass.rs\|hourglass.rs]] | `DragState { is_dragging, start_position, drag_threshold: 10.0 }`. |
| pause banner | [[src/ui/pause_overlay.rs\|pause_overlay.rs]] | `update_pause_overlay_visibility`. |

Key modules: [[modules/hourglass]], [[modules/pause-overlay]].

## Click vs. drag

`handle_hourglass_click` converts the cursor to world coordinates and, on mouse-up within ~400 px of the hourglass center, checks `DragState.is_dragging`. The flag is set during the press if the cursor moved more than the 10 px `drag_threshold`. A drag calls `hourglass.flip()` + `timer_state.reset()` + start; a plain click toggles `is_running`. Full step-by-step in [[flows/click-vs-drag]].

**Crucially**, the handler ignores clicks that land on controls: it skips if the cursor is over a [[modules/shape-panel|mini-hourglass sprite]] (`MiniHourglass`) or any Bevy UI button (`Interaction != None`). Without this guard, selecting a shape or color would also toggle the timer — a bug that was explicitly fixed (git history: "Stop shape selection from toggling timer pause").

## First-start flip

`handle_timer_start` flips the hourglass **only on the first** not-running→running transition (tracked by a `has_ever_started` `Local`). Resuming from a pause doesn't re-flip. The flag resets when the timer returns to full.

## Pause overlay

`update_pause_overlay_visibility` shows the banner only when the timer is paused **and** has time left **and** had already been started — so it never appears on the fresh app or after completion. See [[modules/pause-overlay]].

## Architecture Decisions

- **Threshold-based click/drag** rather than separate gestures — a single left-button interaction serves both, with 10 px disambiguating intent.
- **Hit-test exclusion of controls** because the world-space hourglass and the control buttons overlap in screen space.

## Flow

See [[flows/click-vs-drag]].

## Open Questions

- The ~400 px click radius is a fixed approximation of the hourglass bounds, not the actual mesh extent; very wide/slim shapes may be slightly over- or under-covered.
