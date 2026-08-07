<!-- wiki:sources: src/ui/timer_panel.rs, src/resources.rs -->

# Timer Duration Controls

## What It Does

Lets the user set the countdown length and control playback explicitly, via a collapsible panel at the bottom of the screen. Fourteen +/- buttons (±1 s through ±1 h) adjust the duration around a live `HH:MM:SS` readout; Start / Pause / Reset control playback.

## User Journey

1. User clicks **"Timer Controls"** to reveal the panel.
2. User taps `+5m`, `-15s`, etc. to dial in a duration; the display updates live.
3. User clicks **Start** (or just clicks the hourglass).
4. User can **Pause** or **Reset** at any time.
5. Clicking "Timer Controls" again hides the panel.

## Implementation

| Component | File | Role |
|-----------|------|------|
| panel + buttons | [[src/ui/timer_panel.rs\|timer_panel.rs]] | Spawns toggle, +/- buttons, Start/Pause/Reset, display. |
| adjustment math | [[src/resources.rs\|resources.rs]] | `add_time` (clamped), `reset`, `format_time`. |
| visibility | [[src/ui/mod.rs\|ui/mod.rs]] | `TimerPanelVisible` resource. |

Key modules: [[modules/timer-panel]], [[modules/resources]].

## Adjustment behavior

Each button carries a `TimeAdjustButton { adjustment }`; pressing it calls `timer_state.add_time(adjustment)`. `add_time` adds to both `duration` and `remaining`, then clamps duration to `0..=86400` s (24 h) and remaining to `0..=duration`. So `-` buttons can't drive the timer below zero, and `+` buttons can't exceed 24 hours. The exact clamp ordering is a tested behavior — see [[modules/resources#TimerState]].

## Collapse/expand

The panel defaults hidden. The toggle button flips `TimerPanelVisible`; `update_timer_panel_visibility` swaps the container's `Display` between `Flex` and `None`. As a small optimization, `update_time_display` skips formatting work entirely while the panel is hidden.

## Architecture Decisions

- The panel is the **explicit** control surface; clicking the hourglass is the **implicit** one ([[features/hourglass-interaction]]). Both mutate the same `TimerState`, so they stay in sync for free.

## Flow

The controls feed directly into the [[flows/countdown-tick|countdown tick]]; there's no separate flow page.

## Open Questions

None.
