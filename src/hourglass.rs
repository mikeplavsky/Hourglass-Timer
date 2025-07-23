use crate::resources::{HourglassConfig, HourglassShape, ShapeMode, TimerState};
use bevy::prelude::*;
use bevy_hourglass::{
    BulbStyle, Hourglass, HourglassMeshBodyConfig, HourglassMeshBuilder, HourglassMeshPlatesConfig,
    HourglassMeshSandConfig, HourglassPlugin as BevyHourglassPlugin, NeckStyle, SandSplash, SandSplashConfig,
};

pub struct HourglassPlugin;

impl Plugin for HourglassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BevyHourglassPlugin)
            .add_systems(Startup, spawn_hourglass)
            .add_systems(
                Update,
                (
                    update_hourglass_color,
                    update_hourglass_shape,
                    update_morphing_shape,
                    update_hourglass_timer.after(update_morphing_shape),
                    handle_hourglass_click,
                    handle_timer_start,
                ),
            );
    }
}

#[derive(Component)]
pub struct MainHourglass;

#[derive(Component, Default, Clone)]
struct DragState {
    is_dragging: bool,
    start_position: Vec2,
    drag_threshold: f32,
}

impl DragState {
    fn new() -> Self {
        Self {
            is_dragging: false,
            start_position: Vec2::ZERO,
            drag_threshold: 10.0, // Minimum distance in pixels to consider it a drag
        }
    }
}

// Helper function to create main hourglass configurations for different shapes
fn get_main_shape_config(
    shape: HourglassShape,
) -> (HourglassMeshBodyConfig, HourglassMeshPlatesConfig) {
    let base_height = 400.0; // Full size for main hourglass

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
                width: 400.0,
                height: 10.0,
                ..Default::default()
            },
        ),
        HourglassShape::Modern => (
            HourglassMeshBodyConfig {
                total_height: base_height,
                bulb_style: BulbStyle::Circular {
                    curvature: 0.0,
                    width_factor: 1.0,
                    curve_resolution: 10,
                },
                neck_style: NeckStyle::Straight {
                    width: 12.0,
                    height: 32.0,
                },
                color: Color::srgba(0.85, 0.95, 1.0, 0.2),
            },
            HourglassMeshPlatesConfig {
                width: 380.0,
                height: 12.0,
                ..Default::default()
            },
        ),
        HourglassShape::Slim => (
            HourglassMeshBodyConfig {
                total_height: base_height * 1.2, // Taller
                bulb_style: BulbStyle::Circular {
                    curvature: 1.5,
                    width_factor: 0.7, // Narrower
                    curve_resolution: 18,
                },
                neck_style: NeckStyle::Curved {
                    curvature: 1.5,
                    width: 12.0, // Thinner neck
                    height: 24.0,
                    curve_resolution: 8,
                },
                color: Color::srgba(0.85, 0.95, 1.0, 0.2),
            },
            HourglassMeshPlatesConfig {
                width: 340.0, // Narrower plates
                height: 8.0,
                ..Default::default()
            },
        ),
        HourglassShape::Wide => (
            HourglassMeshBodyConfig {
                total_height: base_height * 0.8, // Shorter
                bulb_style: BulbStyle::Circular {
                    curvature: 1.0,
                    width_factor: 1.2, // Wider
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
                width: 390.0, // Wider plates
                height: 14.0,
                ..Default::default()
            },
        ),
    }
}

// Helper function to create mini hourglass configurations for different shapes (for UI panels)
pub fn get_mini_shape_config(
    shape: HourglassShape,
) -> (HourglassMeshBodyConfig, HourglassMeshPlatesConfig) {
    let base_height = 25.0; // Smaller size for mini hourglasses

    match shape {
        HourglassShape::Classic => (
            HourglassMeshBodyConfig {
                total_height: base_height,
                bulb_style: BulbStyle::Circular {
                    curvature: 1.0,
                    width_factor: 1.0,
                    curve_resolution: 10, // Lower resolution for performance
                },
                neck_style: NeckStyle::Curved {
                    curvature: 1.0,
                    width: 3.0,
                    height: 4.0,
                    curve_resolution: 5,
                },
                color: Color::srgba(0.85, 0.95, 1.0, 0.2),
            },
            HourglassMeshPlatesConfig {
                width: 25.0,
                height: 2.0,
                ..Default::default()
            },
        ),
        HourglassShape::Modern => (
            HourglassMeshBodyConfig {
                total_height: base_height,
                bulb_style: BulbStyle::Circular {
                    curvature: 0.0,
                    width_factor: 1.0,
                    curve_resolution: 5,
                },
                neck_style: NeckStyle::Straight {
                    width: 2.5,
                    height: 6.0,
                },
                color: Color::srgba(0.85, 0.95, 1.0, 0.2),
            },
            HourglassMeshPlatesConfig {
                width: 22.0,
                height: 2.5,
                ..Default::default()
            },
        ),
        HourglassShape::Slim => (
            HourglassMeshBodyConfig {
                total_height: base_height * 1.2,
                bulb_style: BulbStyle::Circular {
                    curvature: 1.5,
                    width_factor: 0.7,
                    curve_resolution: 8,
                },
                neck_style: NeckStyle::Curved {
                    curvature: 1.5,
                    width: 2.0,
                    height: 5.0,
                    curve_resolution: 4,
                },
                color: Color::srgba(0.85, 0.95, 1.0, 0.2),
            },
            HourglassMeshPlatesConfig {
                width: 20.0,
                height: 1.5,
                ..Default::default()
            },
        ),
        HourglassShape::Wide => (
            HourglassMeshBodyConfig {
                total_height: base_height * 0.8,
                bulb_style: BulbStyle::Circular {
                    curvature: 1.0,
                    width_factor: 1.2,
                    curve_resolution: 10,
                },
                neck_style: NeckStyle::Curved {
                    curvature: 0.7,
                    width: 4.0,
                    height: 3.0,
                    curve_resolution: 6,
                },
                color: Color::srgba(0.85, 0.95, 1.0, 0.2),
            },
            HourglassMeshPlatesConfig {
                width: 28.0,
                height: 3.0,
                ..Default::default()
            },
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
            fill_percent: 0.0, // Start with bottom bulb filled (empty top)
            wall_offset: 4.0,
        })
        .with_sand_splash(SandSplashConfig {
            particle_color: config.color,
            splash_radius: 20.0,
            particle_size: 2.0,
            ..Default::default()
        })
        .with_timing(timer_state.duration)
        .build(&mut commands, &mut meshes, &mut materials);
    commands
        .entity(entity)
        .insert((MainHourglass, DragState::new(), Name::new("Main Hourglass")));
}

fn update_hourglass_color(
    config: Res<HourglassConfig>,
    mut hourglass_query: Query<&mut Hourglass, With<MainHourglass>>,
    mut splash_query: Query<&mut SandSplash, With<MainHourglass>>,
) {
    if config.is_changed() {
        // Update sand color
        for mut hourglass in hourglass_query.iter_mut() {
            hourglass.sand_color = config.color;
        }
        
        // Update particle color for sand splash
        for mut sand_splash in splash_query.iter_mut() {
            sand_splash.config.particle_color = config.color;
        }
    }
}


fn update_hourglass_shape(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<HourglassConfig>,
    timer_state: Res<TimerState>,
    query: Query<(Entity, &Hourglass, &DragState), With<MainHourglass>>,
) {
    // Only handle static shape mode, and only recreate hourglass if shape type changed (not color changes)
    if config.is_changed() && config.shape_mode == ShapeMode::Static {
        // Preserve current hourglass state and drag state
        let (
            _current_upper,
            _current_lower,
            _current_running,
            _current_remaining,
            current_drag_state,
        ) = if let Ok((_, hourglass, drag_state)) = query.single() {
            (
                hourglass.upper_chamber,
                hourglass.lower_chamber,
                hourglass.running,
                hourglass.remaining_time,
                drag_state.clone(),
            )
        } else {
            (0.0, 1.0, false, timer_state.duration, DragState::new())
        };

        // Despawn the old hourglass
        for (entity, _, _) in query.iter() {
            commands.entity(entity).despawn();
        }

        // Calculate correct fill percentage based on timer state
        // fill_percent 1.0 = top chamber full, 0.0 = bottom chamber full
        let fill_percent = if timer_state.duration > 0.0 {
            timer_state.remaining / timer_state.duration
        } else {
            1.0
        };

        // Spawn a new hourglass with the new shape and preserved state
        let (body_config, plates_config) = get_main_shape_config(config.shape_type);

        let entity = HourglassMeshBuilder::new(Transform::from_xyz(0.0, 0.0, 0.0))
            .with_body(body_config)
            .with_plates(plates_config)
            .with_sand(HourglassMeshSandConfig {
                color: config.color,
                fill_percent,
                wall_offset: 4.0,
            })
            .with_sand_splash(SandSplashConfig {
                particle_color: config.color,
                splash_radius: 20.0,
                particle_size: 2.0,
                ..Default::default()
            })
            .with_timing(timer_state.duration)
            .build(&mut commands, &mut meshes, &mut materials);

        commands.entity(entity).insert((
            MainHourglass,
            current_drag_state, // Use the preserved drag state
            Name::new("Main Hourglass"),
        ));

        // Note: State will be restored by update_hourglass_timer system
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

        // Always update chamber levels based on timer state, regardless of running state
        if timer_state.duration > 0.0 && !hourglass.flipping {
            let progress = timer_state.remaining / timer_state.duration;
            hourglass.upper_chamber = progress; // Amount of time remaining
            hourglass.lower_chamber = 1.0 - progress; // Amount of time elapsed
        }
    }
}

fn handle_hourglass_click(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera>>,
    mut hourglass_query: Query<(&Transform, &mut DragState, &mut Hourglass), With<MainHourglass>>,
    mut timer_state: ResMut<TimerState>,
) {
    if let Ok(window) = windows.single() {
        if let Some(cursor_position) = window.cursor_position() {
            if let Ok((camera, camera_transform)) = camera_query.single() {
                if let Ok((hourglass_transform, mut drag_state, mut hourglass)) =
                    hourglass_query.single_mut()
                {
                    // Convert screen coordinates to world coordinates
                    if let Ok(world_position) =
                        camera.viewport_to_world_2d(camera_transform, cursor_position)
                    {
                        // Check if interaction is within hourglass bounds (approximate 400x400 area)
                        let hourglass_pos = hourglass_transform.translation.truncate();
                        let distance = world_position.distance(hourglass_pos);

                        if distance < 400.0 {
                            // Larger area to cover most of the hourglass
                            // Handle mouse down - start potential drag
                            if mouse_input.just_pressed(MouseButton::Left) {
                                drag_state.start_position = cursor_position;
                                drag_state.is_dragging = false;
                            }

                            // Handle mouse movement during press - detect drag
                            if mouse_input.pressed(MouseButton::Left) && !drag_state.is_dragging {
                                let drag_distance =
                                    cursor_position.distance(drag_state.start_position);
                                if drag_distance > drag_state.drag_threshold {
                                    drag_state.is_dragging = true;
                                }
                            }

                            // Handle mouse up - complete action
                            if mouse_input.just_released(MouseButton::Left) {
                                if drag_state.is_dragging {
                                    // Drag detected - flip and reset hourglass
                                    if hourglass.can_flip() {
                                        // Immediately set chambers to initial state (all sand in bottom)
                                        hourglass.upper_chamber = 0.0;
                                        hourglass.lower_chamber = 1.0;

                                        // Then trigger the flip animation
                                        hourglass.flip();
                                        timer_state.reset();

                                        // Start the timer automatically after flip
                                        timer_state.is_running = true;
                                    }
                                } else {
                                    // Simple click - toggle pause/play
                                    if timer_state.is_running {
                                        // Pause the timer if it's running
                                        timer_state.is_running = false;
                                    } else {
                                        // Start the timer if it's not running
                                        timer_state.is_running = true;
                                    }
                                }

                                // Reset drag state
                                drag_state.is_dragging = false;
                                drag_state.start_position = Vec2::ZERO;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn update_morphing_shape(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<HourglassConfig>,
    timer_state: Res<TimerState>,
    time: Res<Time>,
    query: Query<(Entity, &Hourglass, &DragState), With<MainHourglass>>,
    mut last_update_time: Local<f32>,
) {
    // Only handle morphing shape mode, and throttle updates to avoid excessive recreation
    if config.shape_mode == ShapeMode::Morphing {
        let current_time = time.elapsed_secs();
        if current_time - *last_update_time < 0.01 {
            return;
        }
        *last_update_time = current_time;

        // Preserve current hourglass state and drag state
        let (
            _current_upper,
            _current_lower,
            _current_running,
            _current_remaining,
            current_flipping,
            current_drag_state,
        ) = if let Ok((_, hourglass, drag_state)) = query.single() {
            (
                hourglass.upper_chamber,
                hourglass.lower_chamber,
                hourglass.running,
                hourglass.remaining_time,
                hourglass.flipping,
                drag_state.clone(),
            )
        } else {
            (
                0.0,
                1.0,
                timer_state.is_running,
                timer_state.remaining,
                false,
                DragState::new(),
            )
        };

        // Don't interrupt the hourglass if it's currently flipping
        if current_flipping {
            return;
        }

        // Cycle through shapes over time (complete cycle every 8 seconds)
        let cycle_time = 8.0;
        let t = (current_time % cycle_time) / cycle_time;

        // Create morphed shape parameters
        let (body_config, plates_config) = get_morphed_shape_config(t);

        // Despawn the old hourglass
        for (entity, _, _) in query.iter() {
            commands.entity(entity).despawn();
        }

        // Calculate correct fill percentage based on timer state
        // fill_percent 1.0 = top chamber full, 0.0 = bottom chamber full
        let fill_percent = if timer_state.duration > 0.0 {
            timer_state.remaining / timer_state.duration
        } else {
            1.0
        };

        // Spawn a new hourglass with the morphed shape and correct sand level
        let entity = HourglassMeshBuilder::new(Transform::from_xyz(0.0, 0.0, 0.0))
            .with_body(body_config)
            .with_plates(plates_config)
            .with_sand(HourglassMeshSandConfig {
                color: config.color,
                fill_percent,
                wall_offset: 4.0,
            })
            .with_sand_splash(SandSplashConfig {
                particle_color: config.color,
                splash_radius: 20.0,
                particle_size: 2.0,
                ..Default::default()
            })
            .with_timing(timer_state.duration)
            .build(&mut commands, &mut meshes, &mut materials);

        commands.entity(entity).insert((
            MainHourglass,
            current_drag_state, // Use the preserved drag state
            Name::new("Main Hourglass"),
        ));

        // Note: State will be restored by update_hourglass_timer system
    }
}

// Helper function to create morphed shape configurations
fn get_morphed_shape_config(t: f32) -> (HourglassMeshBodyConfig, HourglassMeshPlatesConfig) {
    // Define the 4 shape configurations
    let shapes = [
        HourglassShape::Classic,
        HourglassShape::Modern,
        HourglassShape::Slim,
        HourglassShape::Wide,
    ];

    // Determine which shapes to interpolate between
    let segment = t * 4.0; // 0-4 range
    let segment_index = segment.floor() as usize % 4;
    let next_index = (segment_index + 1) % 4;
    let local_t = segment - segment.floor(); // 0-1 within the segment

    let shape1 = shapes[segment_index];
    let shape2 = shapes[next_index];

    // Get the base configurations for both shapes
    let (config1, plates1) = get_main_shape_config(shape1);
    let (config2, plates2) = get_main_shape_config(shape2);

    // Interpolate between the configurations
    let interpolated_body = HourglassMeshBodyConfig {
        total_height: lerp_f32(config1.total_height, config2.total_height, local_t),
        bulb_style: interpolate_bulb_style(&config1.bulb_style, &config2.bulb_style, local_t),
        neck_style: interpolate_neck_style(&config1.neck_style, &config2.neck_style, local_t),
        color: Color::srgba(0.85, 0.95, 1.0, 0.2),
    };

    let interpolated_plates = HourglassMeshPlatesConfig {
        width: lerp_f32(plates1.width, plates2.width, local_t),
        height: lerp_f32(plates1.height, plates2.height, local_t),
        ..Default::default()
    };

    (interpolated_body, interpolated_plates)
}

// Helper functions for interpolation
fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn interpolate_bulb_style(style1: &BulbStyle, style2: &BulbStyle, t: f32) -> BulbStyle {
    match (style1, style2) {
        (
            BulbStyle::Circular {
                curvature: c1,
                width_factor: w1,
                curve_resolution: r1,
            },
            BulbStyle::Circular {
                curvature: c2,
                width_factor: w2,
                curve_resolution: r2,
            },
        ) => BulbStyle::Circular {
            curvature: lerp_f32(*c1, *c2, t),
            width_factor: lerp_f32(*w1, *w2, t),
            curve_resolution: (lerp_f32(*r1 as f32, *r2 as f32, t) as usize).max(5),
        },
        // If styles are different types, just switch at halfway point
        (style1, style2) => {
            if t < 0.5 {
                style1.clone()
            } else {
                style2.clone()
            }
        }
    }
}

fn interpolate_neck_style(style1: &NeckStyle, style2: &NeckStyle, t: f32) -> NeckStyle {
    match (style1, style2) {
        (
            NeckStyle::Curved {
                curvature: c1,
                width: w1,
                height: h1,
                curve_resolution: r1,
            },
            NeckStyle::Curved {
                curvature: c2,
                width: w2,
                height: h2,
                curve_resolution: r2,
            },
        ) => NeckStyle::Curved {
            curvature: lerp_f32(*c1, *c2, t),
            width: lerp_f32(*w1, *w2, t),
            height: lerp_f32(*h1, *h2, t),
            curve_resolution: (lerp_f32(*r1 as f32, *r2 as f32, t) as usize).max(3),
        },
        (
            NeckStyle::Straight {
                width: w1,
                height: h1,
            },
            NeckStyle::Straight {
                width: w2,
                height: h2,
            },
        ) => NeckStyle::Straight {
            width: lerp_f32(*w1, *w2, t),
            height: lerp_f32(*h1, *h2, t),
        },
        // Mixed types - convert straight to curved for interpolation
        (
            NeckStyle::Straight {
                width: w1,
                height: h1,
            },
            NeckStyle::Curved {
                curvature: c2,
                width: w2,
                height: h2,
                curve_resolution: r2,
            },
        ) => NeckStyle::Curved {
            curvature: lerp_f32(0.0, *c2, t),
            width: lerp_f32(*w1, *w2, t),
            height: lerp_f32(*h1, *h2, t),
            curve_resolution: *r2,
        },
        (
            NeckStyle::Curved {
                curvature: c1,
                width: w1,
                height: h1,
                curve_resolution: r1,
            },
            NeckStyle::Straight {
                width: w2,
                height: h2,
            },
        ) => NeckStyle::Curved {
            curvature: lerp_f32(*c1, 0.0, t),
            width: lerp_f32(*w1, *w2, t),
            height: lerp_f32(*h1, *h2, t),
            curve_resolution: *r1,
        },
    }
}

fn handle_timer_start(
    timer_state: Res<TimerState>,
    mut hourglass_query: Query<&mut Hourglass, With<MainHourglass>>,
    mut last_running_state: Local<bool>,
    mut has_ever_started: Local<bool>,
) {
    // Check if timer state changed from not running to running
    if timer_state.is_running && !*last_running_state {
        // Only flip on the very first start (when timer hasn't been started before)
        if !*has_ever_started {
            for mut hourglass in hourglass_query.iter_mut() {
                if hourglass.can_flip() {
                    hourglass.flip();
                }
            }
            *has_ever_started = true;
        }
        // If resuming from pause, don't flip - just continue
    }

    // Reset the "has ever started" flag when timer is reset (remaining == duration)
    if timer_state.remaining >= timer_state.duration && !timer_state.is_running {
        *has_ever_started = false;
    }

    *last_running_state = timer_state.is_running;
}
