<!-- wiki:sources: src/hourglass.rs, src/ui/shape_panel.rs, src/resources.rs -->

# Shape Morphing

## What It Does

A toggle (`∞` button) that makes the hourglass **continuously morph** through all four shapes, smoothly interpolating from Classic → Modern → Slim → Wide → Classic on an 8-second loop, instead of holding one fixed shape.

## User Journey

1. User clicks the **`∞`** button in the shape row.
2. The main hourglass begins smoothly changing silhouette, cycling through the four shapes forever.
3. The `∞` button scales up (1.15×) to show morphing is active.
4. Clicking `∞` again, or picking a specific shape, returns to static mode.

## Implementation

| Component | File | Role |
|-----------|------|------|
| toggle | [[src/ui/shape_panel.rs\|shape_panel.rs]] | `handle_morphing_button_clicks` flips `ShapeMode`. |
| animation | [[src/hourglass.rs\|hourglass.rs]] | `update_morphing_shape` rebuilds each tick with an interpolated shape. |
| interpolation | [[src/hourglass.rs\|hourglass.rs]] | `get_morphed_shape_config`, `lerp_f32`, `interpolate_bulb_style`, `interpolate_neck_style`. |
| state | [[src/resources.rs\|resources.rs]] | `shape_mode: ShapeMode::{Static,Morphing}`. |

Key modules: [[modules/hourglass]], [[modules/shape-panel]].

## How the morph is computed

`update_morphing_shape` runs only in `ShapeMode::Morphing`, throttled to ~every 0.01 s. Each tick it computes a normalized cycle position `t = (elapsed_secs % 8.0) / 8.0` and calls `get_morphed_shape_config(t)`:

- The 4 shapes form a ring. `t` is scaled to `0..4`; `segment_index` picks the current shape, `next_index` the following one, and `local_t` is the fraction within that segment.
- `total_height`, plate width/height, and the bulb/neck styles are interpolated between the two bracketing shapes.

The interpolation helpers handle the awkward cases: `interpolate_bulb_style` lerps two `Circular` bulbs (curve resolution floored at 5) but hard-switches mismatched variants at the halfway point; `interpolate_neck_style` lerps `Curved↔Curved` and `Straight↔Straight`, and for mixed pairs keeps a `Curved` result while ramping curvature from/to 0 (so a straight neck smoothly gains/loses curve). These are the most heavily unit-tested functions in the codebase.

Like all shape changes, morphing uses the **recreate-on-change** rebuild and respects the "don't interrupt a flip" guard. See [[flows/appearance-recreation]] and [[patterns#Recreate-on-change rendering]].

## Architecture Decisions

- **Per-tick rebuild** (throttled) rather than mesh deformation, because `bevy_hourglass` builds a fresh mesh from config — morphing is just feeding it interpolated config each tick.
- **Pure interpolation helpers** extracted so the morph math is testable without rendering.

## Flow

See [[flows/appearance-recreation]].

## Open Questions

- Throttle is ~0.01 s (≈100 Hz), effectively per-frame; the comment says "every 0.1 s (10 FPS)" but the constant is `0.01`. The discrepancy between comment and value is a minor latent inconsistency.
