# Testing Instructions for Sand Particle Fix

## Issue
When Random or Rainbow color modes are selected for the hourglass, sand particles become invisible or retain old colors.

## Fix Applied
Modified the `update_hourglass_color` function in `src/hourglass.rs` to recreate the hourglass entity for Random and Rainbow color modes, ensuring both sand color and particle colors are updated properly.

## Manual Testing Steps

1. **Build and run the application:**
   ```bash
   cargo run
   ```

2. **Test Static Color Mode (should work as before):**
   - Click on any solid color button in the color panel
   - Start the timer by clicking the hourglass
   - Verify that sand particles are visible and match the selected color

3. **Test Random Color Mode (this was broken, now fixed):**
   - Click the Random Color button (with colored squares)
   - Start the timer by clicking the hourglass
   - Verify that sand particles are visible and match the randomly generated color
   - Click Random Color button again to generate a new color
   - Verify particles update to the new color

4. **Test Rainbow Color Mode (this was broken, now fixed):**
   - Click the Rainbow Color button (with rainbow stripes)
   - Start the timer by clicking the hourglass
   - Verify that sand particles are visible and continuously change color
   - The particles should cycle through the rainbow colors over time

## Expected Results
- **Before fix:** Random and Rainbow modes would show invisible or wrong-colored particles
- **After fix:** All color modes show properly colored, visible sand particles

## Technical Details
- Static mode uses efficient sand color updates without recreating the hourglass
- Random and Rainbow modes recreate the hourglass to update both sand and particle colors
- The fix preserves timer state and drag interaction state during recreation