use bevy::prelude::*;
use hourglass_timer::resources::{ColorMode, HourglassConfig};

#[test]
fn test_color_mode_behavior() {
    // Test that color modes are properly defined
    let static_mode = ColorMode::Static;
    let random_mode = ColorMode::Random;
    let rainbow_mode = ColorMode::Rainbow;
    
    // Test default config
    let config = HourglassConfig::default();
    assert_eq!(config.color_mode, ColorMode::Static);
    
    // Test color changes for different modes
    let mut config = HourglassConfig::default();
    
    // Test static mode
    config.color_mode = static_mode;
    config.color = Color::srgb(1.0, 0.0, 0.0); // Red
    assert_eq!(config.color_mode, ColorMode::Static);
    
    // Test random mode
    config.color_mode = random_mode;
    assert_eq!(config.color_mode, ColorMode::Random);
    
    // Test rainbow mode
    config.color_mode = rainbow_mode;
    assert_eq!(config.color_mode, ColorMode::Rainbow);
}

#[test]
fn test_color_mode_equality() {
    // Test that color modes can be compared
    assert_eq!(ColorMode::Static, ColorMode::Static);
    assert_eq!(ColorMode::Random, ColorMode::Random);
    assert_eq!(ColorMode::Rainbow, ColorMode::Rainbow);
    
    assert_ne!(ColorMode::Static, ColorMode::Random);
    assert_ne!(ColorMode::Random, ColorMode::Rainbow);
    assert_ne!(ColorMode::Rainbow, ColorMode::Static);
}