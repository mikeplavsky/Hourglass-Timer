<!-- wiki:sources: src/ui/timer_panel.rs -->

# Timer Panel

## Responsibility

The collapsible control panel at the bottom of the screen. A "Timer Controls" toggle button reveals a row of +/- duration adjusters flanking a live `HH:MM:SS` display, plus Start / Pause / Reset buttons. This is the explicit-control counterpart to clicking the hourglass directly ([[features/hourglass-interaction]]).

## Where It Lives

[[src/ui/timer_panel.rs|src/ui/timer_panel.rs]]

## Systems (registered by `TimerPanelPlugin`)

| System | Schedule | Role |
|--------|----------|------|
| `spawn_timer_controls` | `PostStartup` | Build toggle button + hidden controls container under `BottomTimerMarker`. |
| `handle_timer_buttons` | `Update` | +/- buttons → `timer_state.add_time(adjustment)`. |
| `handle_control_buttons` | `Update` | Start / Pause / Reset. |
| `handle_toggle_button` | `Update` | Flip `TimerPanelVisible`. |
| `update_timer_panel_visibility` | `Update` | Show/hide the controls container when visibility changes. |
| `update_time_display` | `Update` | Refresh the `HH:MM:SS` text (only while visible). |

## Layout

`spawn_timer_controls` builds two pieces under `BottomTimerMarker`:

1. A **toggle button** ("Timer Controls"), always visible.
2. A **controls container** (`TimerControlsContainer`), starting `Display::None`. `spawn_timer_controls_content` fills it with two rows: the time-adjust row (7 negative buttons, the `TimeDisplay`, 7 positive buttons) and the control row (Start/Pause/Reset).

The adjustment buttons carry a `TimeAdjustButton { adjustment }` with values from ±1 s up to ±1 h: `-1h, -15m, -5m, -1m, -15s, -5s, -1s` and the positive mirror.

## Behavior details

- **`handle_timer_buttons`** calls `add_time`, which clamps duration to `0..=24h` and remaining to `0..=duration` (see [[modules/resources#TimerState]]). All three interaction states (Pressed/Hovered/None) also restyle the button background.
- **`handle_control_buttons`** uses three disjoint queries (`Without` filters keep Start/Pause/Reset queries non-overlapping). Start sets `is_running = true` (no-op if already running); Pause sets it false; Reset calls `timer_state.reset()`.
- **`update_timer_panel_visibility`** only runs its body when `TimerPanelVisible.is_changed()`, flipping the container's `Display` between `Flex` and `None`.
- **`update_time_display`** is gated on `panel_visible.0` — it skips formatting work entirely when the panel is hidden. The display starts at the literal `"00:03:00"` placeholder, replaced once the timer ticks.

## Features Supported

- [[features/timer-duration-controls]] — the +/- buttons and Start/Pause/Reset.
- [[features/countdown-timer]] — the time display + control buttons.

## Dependencies

- `bevy` — UI nodes, `Button`, `Interaction`, `RelatedSpawnerCommands`.
- [[modules/resources]] — `TimerState`.
- [[modules/ui-layout]] — `BottomTimerMarker`, `TimerPanelVisible`.

## Used By

Bevy runtime. Shares `TimerState` with [[modules/timer]], [[modules/hourglass]], [[modules/pause-overlay]].

## Tests

11 headless-`App` tests. Each spawns a single pressed button (`Interaction::Pressed` set explicitly, since `Button`'s required `Interaction` defaults to `None`) or a display/container node, runs one `app.update()`, and asserts on the resulting `TimerState` / `TimerPanelVisible` / `Node`. Covered: time-adjust add & subtract, the Start/Pause/Reset control buttons (one app each, since their outcomes conflict), the toggle button both directions, panel show/hide, and the time display updating only when the panel is visible. The underlying `add_time`/`reset`/`format_time` arithmetic is also tested in [[modules/resources]]. See [[references/test-coverage#timer_panel.rs]].

## Related Pages

- [[features/timer-duration-controls]]
- [[modules/resources]]
