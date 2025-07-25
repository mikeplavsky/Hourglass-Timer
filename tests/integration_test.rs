use bevy::prelude::*;
use hourglass_timer::resources::{ColorMode, HourglassConfig, HourglassShape, ShapeMode};

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

#[test]
fn test_rainbow_static_combination() {
    // Test the specific combination that was causing issues
    let mut config = HourglassConfig::default();

    // Set to rainbow color mode with static shapes
    config.color_mode = ColorMode::Rainbow;
    config.shape_mode = ShapeMode::Static;
    config.shape_type = HourglassShape::Classic;

    // Verify the configuration
    assert_eq!(config.color_mode, ColorMode::Rainbow);
    assert_eq!(config.shape_mode, ShapeMode::Static);
    assert_eq!(config.shape_type, HourglassShape::Classic);

    // Test that changing only color doesn't affect shape settings
    let original_shape_type = config.shape_type;
    let original_shape_mode = config.shape_mode;

    config.color = Color::srgb(0.5, 0.8, 0.2); // Change color

    assert_eq!(config.shape_type, original_shape_type);
    assert_eq!(config.shape_mode, original_shape_mode);
    assert_eq!(config.color_mode, ColorMode::Rainbow);
}

#[test]
fn test_shape_change_tracking() {
    // Test that we can detect when shape type or mode changes vs color changes
    let mut config1 = HourglassConfig::default();
    let mut config2 = HourglassConfig::default();

    // Initially identical
    assert_eq!(config1.shape_type, config2.shape_type);
    assert_eq!(config1.shape_mode, config2.shape_mode);

    // Change only color in config1
    config1.color = Color::srgb(1.0, 0.0, 0.0);

    // Shape-related fields should still be equal
    assert_eq!(config1.shape_type, config2.shape_type);
    assert_eq!(config1.shape_mode, config2.shape_mode);

    // Change shape type in config1
    config1.shape_type = HourglassShape::Modern;

    // Now shape should be different
    assert_ne!(config1.shape_type, config2.shape_type);

    // Change shape mode in config2
    config2.shape_mode = ShapeMode::Morphing;

    // Now shape mode should be different
    assert_ne!(config1.shape_mode, config2.shape_mode);
}
