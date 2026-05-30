<!-- wiki:sources: src/timer.rs, src/resources.rs, src/hourglass.rs, src/ui/timer_panel.rs -->

# Countdown Timer

## What It Does

The core feature: a countdown that decrements in real time and drives the hourglass's sand level. When running, sand "flows" from the top chamber to the bottom; when it reaches zero, the timer stops on its own.

## User Journey

1. App opens with a 3-minute timer, paused, hourglass full.
2. User starts it (click the hourglass, or the Start button).
3. Each frame, `remaining` decreases; the hourglass's chambers update to match.
4. At zero, the timer stops and the hourglass is empty.

## Implementation

| Component | File | Role |
|-----------|------|------|
| countdown logic | [[src/timer.rs\|timer.rs]] | `update_timer` + pure `tick_countdown` decrement the timer. |
| state | [[src/resources.rs\|resources.rs]] | `TimerState { duration, remaining, is_running }`. |
| visual sync | [[src/hourglass.rs\|hourglass.rs]] | `update_hourglass_timer` copies `TimerState` → `Hourglass` chambers each frame. |
| display | [[src/ui/timer_panel.rs\|timer_panel.rs]] | `update_time_display` shows `HH:MM:SS`. |

Key modules: [[modules/timer]], [[modules/resources]], [[modules/hourglass]].

## How state becomes pixels

The countdown logic and the rendering are deliberately decoupled. [[modules/timer|`update_timer`]] only mutates the `TimerState` resource — it never touches the hourglass. A separate system, [[modules/hourglass#update_hourglass_timer|`update_hourglass_timer`]], reads `TimerState` every frame and writes the `Hourglass` component's `total_time`, `remaining_time`, `running`, and chamber fill (`upper_chamber = remaining / duration`). This keeps the arithmetic pure and unit-testable while the visual concerns stay in the hourglass module.

## Architecture Decisions

- **Pure tick function** extracted from the system so the decrement/clamp/stop behavior is testable without a Bevy `App`. See [[architecture/overview#Key Design Decisions]].
- **Clamp-to-zero**: `tick_countdown` never produces a negative remaining, so the `HH:MM:SS` display never shows negative time even though `format_time` itself doesn't guard against it.

## Flow

See [[flows/countdown-tick]] for the full per-frame sequence.

## Open Questions

None — this is the most straightforward and best-tested part of the app.
