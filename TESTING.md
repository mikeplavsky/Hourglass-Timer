# Testing Instructions for Color Change Fixes

## Issues Fixed

### Issue 1: Sand Particle Visibility (Previous Fix)
When Random or Rainbow color modes are selected for the hourglass, sand particles become invisible or retain old colors.

### Issue 2: Static Color Changes Not Working (Current Fix)
When clicking static color buttons (blue, red, green, etc.), only sand particles would change color but the main sand body would remain the old color. Rainbow colors worked fine.

## Fixes Applied

### Previous Fix
Modified the `update_hourglass_color` function in `src/hourglass.rs` to recreate the hourglass entity for Random and Rainbow color modes, ensuring both sand color and particle colors are updated properly.

### Current Fix (Issue #4)
Modified the `update_hourglass_shape` function in `src/hourglass.rs` to:
1. Track color mode changes and trigger recreation when switching between modes
2. Always recreate the hourglass for static color changes to ensure proper color application
3. Maintain efficient handling for rainbow mode updates

## Manual Testing Steps

1. **Build and run the application:**
   ```bash
   cargo run
   ```

2. **Test Static Color Mode (Issue #4 - this was broken, now fixed):**
   - Click on any solid color button in the color panel (blue, red, green, etc.)
   - **Expected:** Both the sand body AND sand particles should immediately change to the selected color
   - **Before fix:** Only particles changed color, sand body remained old color
   - **After fix:** Both sand body and particles change to the selected color
   - Start the timer by clicking the hourglass to see sand flowing with correct color

3. **Test Random Color Mode (previous issue - should continue working):**
   - Click the Random Color button (with colored squares)
   - Start the timer by clicking the hourglass
   - Verify that sand particles are visible and match the randomly generated color
   - Click Random Color button again to generate a new color
   - Verify particles update to the new color

4. **Test Rainbow Color Mode (should continue working):**
   - Click the Rainbow Color button (with rainbow stripes)
   - Start the timer by clicking the hourglass
   - Verify that sand particles are visible and continuously change color
   - The particles should cycle through the rainbow colors over time

5. **Test Color Mode Transitions (Issue #4 fix):**
   - Start with a static color (e.g., blue)
   - Switch to rainbow mode - should work smoothly
   - Switch back to a different static color (e.g., red) - should update immediately
   - Switch to random mode - should work properly

## Expected Results

### Issue #4 Fix (Static Color Changes)
- **Before fix:** Clicking static color buttons (blue, red, green, etc.) only changed particle colors, leaving the main sand body with the old color
- **After fix:** Clicking static color buttons immediately updates both the sand body and particles to the selected color

### Previous Fix (Random/Rainbow Particle Visibility)  
- **Before previous fix:** Random and Rainbow modes would show invisible or wrong-colored particles
- **After previous fix:** All color modes show properly colored, visible sand particles

## Technical Details
- **Static mode:** Now recreates the hourglass to ensure both sand body and particles get the correct color
- **Random and Rainbow modes:** Continue to recreate the hourglass to update both sand and particle colors  
- **Color mode transitions:** Properly handled by tracking color mode changes
- **Performance:** The fix preserves timer state and drag interaction state during recreation
- **No regressions:** Rainbow mode continues to work with throttled updates for smooth color transitions

## Chrome Extension Overturn Gesture

1. Build the extension with `./build_extension.sh`, load `dist/chrome-extension` from `chrome://extensions`, and open its side panel.
2. Start the timer, then press on the main hourglass, drag more than 10 px, and release while still over the hourglass.
   - **Expected:** The hourglass flips, the timer returns to its configured duration, and the countdown starts immediately.
3. Repeat, but make a longer swipe that leaves the scaled hourglass hit area before release. Release elsewhere in the black canvas, including over the appearance or timer-control region.
   - **Expected:** The captured gesture still flips and restarts exactly once.
4. Click the hourglass without dragging.
   - **Expected:** The timer only toggles pause/play; it does not restart.
5. Begin a drag outside the main hourglass and move onto it before release.
   - **Expected:** No hourglass click or restart is triggered.
6. Click each color, shape, and timer control.
   - **Expected:** The control performs its own action without also toggling the hourglass through the gesture handler.
