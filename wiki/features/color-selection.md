<!-- wiki:sources: src/ui/color_panel.rs, src/hourglass.rs, src/resources.rs -->

# Color Selection

## What It Does

Lets the user recolor the hourglass sand three ways: pick one of 8 fixed **swatches**, roll a **random** color, or enable a continuously **cycling rainbow**. Both the sand body and the falling particles update.

## User Journey

1. User clicks a **swatch** → sand turns that color (static).
2. Or clicks the **`?`-with-squares** button → sand turns a random, noticeably-different color.
3. Or clicks the **rainbow stripes** button → sand continuously cycles through the spectrum.
4. In every case, the countdown **restarts from full and starts running**.

## Implementation

| Component | File | Role |
|-----------|------|------|
| buttons + modes | [[src/ui/color_panel.rs\|color_panel.rs]] | Swatches, random, rainbow; `update_rainbow_color` animates the hue. |
| state | [[src/resources.rs\|resources.rs]] | `color`, `color_mode`, `COLOR_PALETTE`. |
| apply to sand | [[src/hourglass.rs\|hourglass.rs]] | `update_hourglass_color` (in-place) + the rebuild path for full updates. |
| mini sync | [[src/ui/shape_panel.rs\|shape_panel.rs]] | `update_mini_hourglass_colors` recolors the shape selectors too. |

Key modules: [[modules/color-panel]], [[modules/hourglass]].

## The three modes

- **Static** — one of `COLOR_PALETTE`'s 8 swatches, set on click.
- **Random** — `pick_distinct_color` re-rolls until the new color is ≥ `0.3²` squared-RGB distance from the current one, guaranteeing a visible change.
- **Rainbow** — `update_rainbow_color` sets `color = hsl_to_rgb(rainbow_hue(elapsed), 1.0, 0.5)` every frame; one full hue cycle per 6 s.

## How the color reaches the sand

There are two paths, which is a subtle part of the design:

1. **In place**: [[modules/hourglass#update_hourglass_color|`update_hourglass_color`]] sets `hourglass.sand_color` and the `SandSplash` particle color whenever config changes.
2. **Full rebuild**: in-place updates alone left the *sand body mesh* stale on static color changes, so [[modules/hourglass#update_hourglass_shape|`update_hourglass_shape`]] also tracks `color_mode` and **rebuilds the hourglass** on static/random color changes (rainbow is throttled to ~every 0.01 s to keep particles visible). This two-path arrangement is the fix documented in [[TESTING.md|TESTING.md]]. See [[flows/appearance-recreation]].

## Side effect: restarts the timer

All three color buttons call `timer_state.reset()` then start the timer. **Picking a color restarts the countdown.** This is intentional and documented in code comments; the continuous rainbow hue updates do *not* restart it — only the button press does.

## Architecture Decisions

- **Distinct-color re-roll** so "random" never feels like nothing happened. See [[architecture/overview#Key Design Decisions]].
- **Rebuild for static colors** to work around `bevy_hourglass` not refreshing the sand body mesh on a bare `sand_color` change.

## Flow

See [[flows/appearance-recreation]].

## Open Questions

- The static-color rebuild is heavier than an in-place update; see the Open Question in [[modules/hourglass]].
