use bevy::prelude::*;

/// Resource to track the current hourglass configuration
#[derive(Resource, Debug, Clone)]
pub struct HourglassConfig {
    pub color: Color,
    pub shape_type: HourglassShape,
    pub color_mode: ColorMode,
    pub shape_mode: ShapeMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMode {
    Static,
    Random,
    Rainbow,
}

impl Default for HourglassConfig {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.8, 0.6, 0.2), // Sandy color
            shape_type: HourglassShape::Classic,
            color_mode: ColorMode::Static,
            shape_mode: ShapeMode::Static,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HourglassShape {
    Classic,
    Modern,
    Slim,
    Wide,
    // Add more shapes as needed
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShapeMode {
    Static,
    Morphing,
}

/// Resource to manage the countdown timer
#[derive(Resource, Debug)]
pub struct TimerState {
    pub duration: f32,  // Total duration in seconds
    pub remaining: f32, // Remaining time in seconds
    pub is_running: bool,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            duration: 180.0, // Default 3 minutes
            remaining: 180.0,
            is_running: false,
        }
    }
}

impl TimerState {
    pub fn reset(&mut self) {
        self.remaining = self.duration;
        self.is_running = false;
    }

    pub fn add_time(&mut self, seconds: f32) {
        self.duration += seconds;
        self.remaining += seconds;
        // Clamp to reasonable values
        self.duration = self.duration.clamp(0.0, 3600.0 * 24.0); // Max 24 hours
        self.remaining = self.remaining.max(0.0).min(self.duration);
    }

    pub fn format_time(&self) -> String {
        let total_seconds = self.remaining as i32;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    }
}

/// Set by color/shape UI handlers to request a flip animation on the next
/// (re)spawned main hourglass. Consumed by `apply_pending_flip`.
#[derive(Resource, Default)]
pub struct PendingFlip(pub bool);

// Color palette for the color selector
pub const COLOR_PALETTE: &[Color] = &[
    Color::srgb(0.0, 0.0, 0.0), // Black
    Color::srgb(1.0, 1.0, 1.0), // White
    Color::srgb(0.1, 0.3, 0.8), // Blue
    Color::srgb(0.8, 0.2, 0.2), // Red
    Color::srgb(0.7, 0.1, 0.8), // Purple
    Color::srgb(0.1, 0.5, 0.1), // Green
    Color::srgb(0.8, 0.8, 0.2), // Yellow
    Color::srgb(0.8, 0.4, 0.0), // Orange
];

#[cfg(test)]
mod tests {
    use super::*;

    fn state(duration: f32, remaining: f32, is_running: bool) -> TimerState {
        TimerState {
            duration,
            remaining,
            is_running,
        }
    }

    #[test]
    fn reset_restores_remaining_and_stops() {
        let mut s = state(120.0, 3.0, true);
        s.reset();
        assert_eq!(s.remaining, 120.0);
        assert!(!s.is_running);
    }

    #[test]
    fn add_time_positive_increases_both() {
        let mut s = state(180.0, 180.0, false);
        s.add_time(60.0);
        assert_eq!(s.duration, 240.0);
        assert_eq!(s.remaining, 240.0);
    }

    #[test]
    fn add_time_negative_clamps_remaining_to_zero() {
        let mut s = state(180.0, 10.0, false);
        s.add_time(-100.0);
        assert_eq!(s.duration, 80.0);
        // 10 - 100 = -90, floored to 0, then min(80) = 0.
        assert_eq!(s.remaining, 0.0);
    }

    #[test]
    fn add_time_clamps_duration_before_remaining() {
        // Duration is clamped to 86400 first, then remaining is min(duration).
        let mut s = state(86_000.0, 86_000.0, false);
        s.add_time(1000.0);
        assert_eq!(s.duration, 86_400.0);
        assert_eq!(s.remaining, 86_400.0);
    }

    #[test]
    fn add_time_clamps_duration_lower_bound() {
        let mut s = state(10.0, 10.0, false);
        s.add_time(-50.0);
        assert_eq!(s.duration, 0.0);
        assert_eq!(s.remaining, 0.0);
    }

    #[test]
    fn add_time_no_clamp_in_normal_range() {
        let mut s = state(100.0, 50.0, false);
        s.add_time(20.0);
        assert_eq!(s.duration, 120.0);
        assert_eq!(s.remaining, 70.0);
    }

    #[test]
    fn format_time_boundaries() {
        assert_eq!(state(0.0, 0.0, false).format_time(), "00:00:00");
        assert_eq!(state(0.0, 59.0, false).format_time(), "00:00:59");
        assert_eq!(state(0.0, 60.0, false).format_time(), "00:01:00");
        assert_eq!(state(0.0, 3599.0, false).format_time(), "00:59:59");
        assert_eq!(state(0.0, 3600.0, false).format_time(), "01:00:00");
        assert_eq!(state(0.0, 3661.0, false).format_time(), "01:01:01");
        assert_eq!(state(0.0, 86_399.0, false).format_time(), "23:59:59");
    }

    #[test]
    fn format_time_truncates_toward_zero() {
        // `as i32` truncates the fractional part.
        assert_eq!(state(0.0, 61.9, false).format_time(), "00:01:01");
    }

    #[test]
    fn format_time_negative_is_not_zero_padded() {
        // Pins a known latent quirk: negative seconds bypass the zero-padding
        // intent. In practice the countdown clamps remaining to 0 upstream, so
        // the UI never displays this — but format_time itself does not guard.
        assert_eq!(state(0.0, -5.0, false).format_time(), "00:00:-5");
    }
}
