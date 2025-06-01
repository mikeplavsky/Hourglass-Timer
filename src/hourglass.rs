use bevy::prelude::*;
use bevy_hourglass::{BulbStyle, Hourglass, HourglassMeshBodyConfig, HourglassMeshBuilder, HourglassMeshPlatesConfig, HourglassMeshSandConfig, HourglassPlugin as BevyHourglassPlugin, NeckStyle, SandSplashConfig};
use crate::resources::{HourglassConfig, HourglassShape, TimerState};

pub struct HourglassPlugin;

impl Plugin for HourglassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BevyHourglassPlugin)
            .add_systems(Startup, spawn_hourglass)
            .add_systems(Update, (
                update_hourglass_color,
                update_hourglass_timer,
                update_hourglass_shape,
            ));
    }
}

#[derive(Component)]
pub struct MainHourglass;

// Helper function to create main hourglass configurations for different shapes
fn get_main_shape_config(shape: HourglassShape) -> (HourglassMeshBodyConfig, HourglassMeshPlatesConfig) {
    let base_height = 200.0; // Full size for main hourglass

    match shape {
        HourglassShape::Classic => (
            HourglassMeshBodyConfig {
                total_height: base_height,
                bulb_style: BulbStyle::Circular {
                    curvature: 1.0,
                    width_factor: 1.0,
                    curve_resolution: 20,
                },
                neck_style: NeckStyle::Curved {
                    curvature: 1.0,
                    width: 14.0,
                    height: 20.0,
                    curve_resolution: 10,
                },
                color: Color::srgba(0.85, 0.95, 1.0, 0.2),
            },
            HourglassMeshPlatesConfig {
                width: 200.0,
                height: 10.0,
                color: Color::srgb(0.6, 0.4, 0.2),
            }
        ),
        HourglassShape::Modern => (
            HourglassMeshBodyConfig {
                total_height: base_height,
                bulb_style: BulbStyle::Circular {
                    curvature: 0.3, // Sharper curves
                    width_factor: 0.9,
                    curve_resolution: 16,
                },
                neck_style: NeckStyle::Straight {
                    width: 12.0,
                    height: 32.0,
                },
                color: Color::srgba(0.85, 0.95, 1.0, 0.2),
            },
            HourglassMeshPlatesConfig {
                width: 180.0,
                height: 12.0,
                color: Color::srgb(0.4, 0.4, 0.6),
            }
        ),
        HourglassShape::Slim => (
            HourglassMeshBodyConfig {
                total_height: base_height * 1.2, // Taller
                bulb_style: BulbStyle::Circular {
                    curvature: 1.2,
                    width_factor: 0.7, // Narrower
                    curve_resolution: 18,
                },
                neck_style: NeckStyle::Curved {
                    curvature: 1.5,
                    width: 10.0, // Thinner neck
                    height: 24.0,
                    curve_resolution: 8,
                },
                color: Color::srgba(0.85, 0.95, 1.0, 0.2),
            },
            HourglassMeshPlatesConfig {
                width: 140.0, // Narrower plates
                height: 8.0,
                color: Color::srgb(0.5, 0.3, 0.6),
            }
        ),
        HourglassShape::Wide => (
            HourglassMeshBodyConfig {
                total_height: base_height * 0.8, // Shorter
                bulb_style: BulbStyle::Circular {
                    curvature: 0.8,
                    width_factor: 1.3, // Wider
                    curve_resolution: 24,
                },
                neck_style: NeckStyle::Curved {
                    curvature: 0.7,
                    width: 20.0, // Thicker neck
                    height: 16.0,
                    curve_resolution: 12,
                },
                color: Color::srgba(0.85, 0.95, 1.0, 0.2),
            },
            HourglassMeshPlatesConfig {
                width: 260.0, // Wider plates
                height: 14.0,
                color: Color::srgb(0.6, 0.3, 0.3),
            }
        ),
    }
}

// Helper function to create mini hourglass configurations for different shapes (for UI panels)
pub fn get_mini_shape_config(shape: HourglassShape) -> (HourglassMeshBodyConfig, HourglassMeshPlatesConfig) {
    let base_height = 40.0; // Smaller size for mini hourglasses

    match shape {
        HourglassShape::Classic => (
            HourglassMeshBodyConfig {
                total_height: base_height,
                bulb_style: BulbStyle::Circular {
                    curvature: 1.0,
                    width_factor: 1.0,
                    curve_resolution: 12, // Lower resolution for performance
                },
                neck_style: NeckStyle::Curved {
                    curvature: 1.0,
                    width: 3.0,
                    height: 4.0,
                    curve_resolution: 6,
                },
                color: Color::srgba(0.85, 0.95, 1.0, 0.3),
            },
            HourglassMeshPlatesConfig {
                width: 30.0,
                height: 2.0,
                color: Color::srgb(0.6, 0.4, 0.2),
            }
        ),
        HourglassShape::Modern => (
            HourglassMeshBodyConfig {
                total_height: base_height,
                bulb_style: BulbStyle::Circular {
                    curvature: 0.3,
                    width_factor: 0.9,
                    curve_resolution: 10,
                },
                neck_style: NeckStyle::Straight {
                    width: 2.5,
                    height: 6.0,
                },
                color: Color::srgba(0.85, 0.95, 1.0, 0.3),
            },
            HourglassMeshPlatesConfig {
                width: 28.0,
                height: 2.5,
                color: Color::srgb(0.4, 0.4, 0.6),
            }
        ),
        HourglassShape::Slim => (
            HourglassMeshBodyConfig {
                total_height: base_height * 1.2,
                bulb_style: BulbStyle::Circular {
                    curvature: 1.2,
                    width_factor: 0.7,
                    curve_resolution: 10,
                },
                neck_style: NeckStyle::Curved {
                    curvature: 1.5,
                    width: 2.0,
                    height: 5.0,
                    curve_resolution: 5,
                },
                color: Color::srgba(0.85, 0.95, 1.0, 0.3),
            },
            HourglassMeshPlatesConfig {
                width: 22.0,
                height: 1.5,
                color: Color::srgb(0.5, 0.3, 0.6),
            }
        ),
        HourglassShape::Wide => (
            HourglassMeshBodyConfig {
                total_height: base_height * 0.8,
                bulb_style: BulbStyle::Circular {
                    curvature: 0.8,
                    width_factor: 1.3,
                    curve_resolution: 14,
                },
                neck_style: NeckStyle::Curved {
                    curvature: 0.7,
                    width: 4.0,
                    height: 3.0,
                    curve_resolution: 7,
                },
                color: Color::srgba(0.85, 0.95, 1.0, 0.3),
            },
            HourglassMeshPlatesConfig {
                width: 38.0,
                height: 3.0,
                color: Color::srgb(0.6, 0.3, 0.3),
            }
        ),
    }
}

fn spawn_hourglass(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<HourglassConfig>,
    timer_state: Res<TimerState>,
) {
    let (body_config, plates_config) = get_main_shape_config(config.shape_type);

    // Create an hourglass with body, plates, and automatic timing using the builder pattern
    let entity = HourglassMeshBuilder::new(Transform::from_xyz(0.0, 0.0, 0.0))
        .with_body(body_config)
        .with_plates(plates_config)
        .with_sand(HourglassMeshSandConfig {
            color: config.color,
            fill_percent: 1.0,
            wall_offset: 4.0,
        })
        .with_sand_splash(SandSplashConfig::default())
        .with_timing(timer_state.duration)
        .build(&mut commands, &mut meshes, &mut materials);
    commands.entity(entity).insert((MainHourglass, Name::new("Main Hourglass")));
}

fn update_hourglass_color(
    config: Res<HourglassConfig>,
    mut query: Query<&mut Hourglass, With<MainHourglass>>,
) {
    if config.is_changed() {
        for mut hourglass in query.iter_mut() {
            hourglass.sand_color = config.color;
        }
    }
}

fn update_hourglass_shape(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<HourglassConfig>,
    timer_state: Res<TimerState>,
    query: Query<Entity, With<MainHourglass>>,
) {
    // Only recreate hourglass if shape specifically changed, not just any config change
    if config.is_changed() {
        // Check if we actually need to recreate (shape change requires recreation)
        // For now, we'll recreate on any config change, but color-only changes
        // will be handled by update_hourglass_color

        // Despawn the old hourglass
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }

        // Spawn a new hourglass with the new shape and current color
        let (body_config, plates_config) = get_main_shape_config(config.shape_type);

        let entity = HourglassMeshBuilder::new(Transform::from_xyz(0.0, 0.0, 0.0))
            .with_body(body_config)
            .with_plates(plates_config)
            .with_sand(HourglassMeshSandConfig {
                color: config.color,
                fill_percent: 1.0,
                wall_offset: 4.0,
            })
            .with_sand_splash(SandSplashConfig::default())
            .with_timing(timer_state.duration)
            .build(&mut commands, &mut meshes, &mut materials);
        commands.entity(entity).insert((MainHourglass, Name::new("Main Hourglass")));
    }
}

fn update_hourglass_timer(
    timer_state: Res<TimerState>,
    mut query: Query<&mut Hourglass, With<MainHourglass>>,
) {
    for mut hourglass in query.iter_mut() {
        hourglass.total_time = timer_state.duration;
        hourglass.remaining_time = timer_state.remaining;
        hourglass.running = timer_state.is_running;

        // Update chamber levels based on remaining time
        // When timer starts (remaining = duration), top should be empty (0.0) and bottom full (1.0)
        // When timer ends (remaining = 0), top should be full (1.0) and bottom empty (0.0)
        if timer_state.duration > 0.0 {
            let progress = timer_state.remaining / timer_state.duration;
            hourglass.upper_chamber = 1.0 - progress;  // Inverted: starts at 0.0
            hourglass.lower_chamber = progress;        // Starts at 1.0
        }
    }
}
