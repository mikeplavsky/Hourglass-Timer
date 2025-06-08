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
