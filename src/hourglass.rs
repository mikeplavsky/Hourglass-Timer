use crate::resources::{
    ColorMode, HourglassConfig, HourglassShape, PendingFlip, ShapeMode, TimerState,
};
use crate::timer::{TimerCommand, TimerSet};
use crate::ui::shape_panel::MiniHourglass;
#[cfg(feature = "chrome_extension")]
use crate::ui::{AppearancePanelVisible, TimerPanelVisible};
use bevy::prelude::*;
use bevy_hourglass::{
    BulbStyle, Hourglass, HourglassMeshBodyConfig, HourglassMeshBuilder, HourglassMeshPlatesConfig,
    HourglassMeshSandConfig, HourglassPlugin as BevyHourglassPlugin, NeckStyle, SandSplash,
    SandSplashConfig,
};

pub struct HourglassPlugin;

impl Plugin for HourglassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BevyHourglassPlugin)
            .init_resource::<PendingFlip>()
            .add_systems(Startup, spawn_hourglass)
            .add_systems(
                Update,
                (
                    apply_pending_flip
                        .before(update_hourglass_shape)
                        .before(update_morphing_shape),
                    update_hourglass_color,
                    update_hourglass_shape,
                    update_morphing_shape,
                    update_hourglass_timer.after(update_morphing_shape),
                    handle_timer_start,
                )
                    .in_set(TimerSet::Observe),
            )
            .add_systems(Update, handle_hourglass_click.in_set(TimerSet::Input));

        #[cfg(feature = "chrome_extension")]
        app.add_systems(
            Update,
            update_sidebar_hourglass_scale.in_set(TimerSet::Observe),
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

/// Whether a world-space point lands inside a button's circular hit area of
/// radius `base_radius * scale_x` centered on `center`. Buttons scale up when
/// hovered, so the radius tracks the current X scale. Boundary is exclusive
/// (a point exactly `base_radius * scale_x` away is a miss).
pub(crate) fn within_click_radius(
    world_pos: Vec2,
    center: Vec2,
    base_radius: f32,
    scale_x: f32,
) -> bool {
    world_pos.distance(center) < base_radius * scale_x
}

/// Whether the pointer has moved far enough from where the press began to count
/// as a drag rather than a click. Boundary is exclusive (a move of exactly
/// `threshold` is still treated as a click).
pub(crate) fn exceeds_drag_threshold(start: Vec2, current: Vec2, threshold: f32) -> bool {
    current.distance(start) > threshold
}

fn main_hourglass_hit_radius(scale: f32) -> f32 {
    if cfg!(feature = "chrome_extension") {
        220.0 * scale.max(0.0)
    } else {
        400.0
    }
}

#[cfg(feature = "chrome_extension")]
fn sidebar_hourglass_scale(
    window_width: f32,
    window_height: f32,
    appearance_open: bool,
    timer_adjustments_open: bool,
) -> f32 {
    let top_reserved = if appearance_open { 92.0 } else { 42.0 };
    let bottom_reserved = if timer_adjustments_open { 190.0 } else { 42.0 };
    let horizontal = (window_width - 24.0) / 400.0;
    let vertical = (window_height - top_reserved - bottom_reserved - 24.0) / 480.0;
    horizontal.min(vertical).clamp(0.35, 1.0)
}

#[cfg(feature = "chrome_extension")]
fn update_sidebar_hourglass_scale(
    windows: Query<&Window>,
    appearance_visible: Res<AppearancePanelVisible>,
    timer_panel_visible: Res<TimerPanelVisible>,
    mut query: Query<&mut Transform, With<MainHourglass>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let scale = sidebar_hourglass_scale(
        window.width(),
        window.height(),
        appearance_visible.0,
        timer_panel_visible.0,
    );
    for mut transform in &mut query {
        transform.scale = Vec3::splat(scale);
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
    time: Res<Time>,
    query: Query<(Entity, &Hourglass, &DragState), With<MainHourglass>>,
    mut last_shape_type: Local<Option<HourglassShape>>,
    mut last_shape_mode: Local<Option<ShapeMode>>,
    mut last_recreation_time: Local<f32>,
    mut last_color_mode: Local<Option<ColorMode>>,
) {
    // Only handle static shape mode
    if config.shape_mode == ShapeMode::Static {
        // Check if shape type or shape mode actually changed
        let shape_changed = last_shape_type.is_none_or(|last| last != config.shape_type);
        let mode_changed = last_shape_mode.is_none_or(|last| last != config.shape_mode);
        let color_mode_changed = last_color_mode.is_none_or(|last| last != config.color_mode);

        // For shape/mode changes, recreate immediately
        if shape_changed || mode_changed || color_mode_changed {
            *last_shape_type = Some(config.shape_type);
            *last_shape_mode = Some(config.shape_mode);
            *last_color_mode = Some(config.color_mode);
            *last_recreation_time = time.elapsed_secs();
        }
        // For color-only changes in rainbow mode, throttle recreation to allow particles but update colors
        else if config.is_changed() && config.color_mode == ColorMode::Rainbow {
            let current_time = time.elapsed_secs();
            // Only recreate every 0.1 seconds (10 FPS) to balance color updates with particle visibility
            if current_time - *last_recreation_time < 0.01 {
                return; // Throttle recreation to prevent particle issues
            }
            *last_recreation_time = current_time;
        }
        // For static color changes, recreate to ensure color is applied properly
        else if config.is_changed() && config.color_mode == ColorMode::Static {
            // Always recreate for static color changes to ensure proper color update
            *last_recreation_time = time.elapsed_secs();
        }
        // For other cases where nothing changed, return early
        else if !shape_changed && !mode_changed && !config.is_changed() {
            return;
        }
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
                false,
                timer_state.duration,
                false,
                DragState::new(),
            )
        };

        // Don't interrupt the hourglass if it's currently flipping
        if current_flipping {
            return;
        }

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
    mut timer_commands: EventWriter<TimerCommand>,
    mini_button_query: Query<
        (&Transform, &Visibility),
        (With<MiniHourglass>, Without<MainHourglass>),
    >,
    ui_interaction_query: Query<&Interaction>,
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
                        // Don't treat clicks on controls as hourglass clicks, otherwise
                        // selecting a shape/color would also toggle the timer's pause state.

                        // Sprite buttons (shape/morphing/random) carry MiniHourglass.
                        let over_mini_button =
                            mini_button_query.iter().any(|(transform, visibility)| {
                                *visibility != Visibility::Hidden
                                    && within_click_radius(
                                        world_position,
                                        transform.translation.truncate(),
                                        30.0,
                                        transform.scale.x,
                                    )
                            });

                        // Bevy UI buttons (color row + timer panel) use Interaction.
                        let over_ui_button = ui_interaction_query
                            .iter()
                            .any(|interaction| *interaction != Interaction::None);

                        if over_mini_button || over_ui_button {
                            return;
                        }

                        // Check if interaction is within hourglass bounds (approximate 400x400 area)
                        let hourglass_pos = hourglass_transform.translation.truncate();
                        let distance = world_position.distance(hourglass_pos);

                        if distance < main_hourglass_hit_radius(hourglass_transform.scale.x) {
                            // Larger area to cover most of the hourglass
                            // Handle mouse down - start potential drag
                            if mouse_input.just_pressed(MouseButton::Left) {
                                drag_state.start_position = cursor_position;
                                drag_state.is_dragging = false;
                            }

                            // Handle mouse movement during press - detect drag
                            if mouse_input.pressed(MouseButton::Left)
                                && !drag_state.is_dragging
                                && exceeds_drag_threshold(
                                    drag_state.start_position,
                                    cursor_position,
                                    drag_state.drag_threshold,
                                )
                            {
                                drag_state.is_dragging = true;
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
                                        timer_commands.write(TimerCommand::Restart);
                                    }
                                } else {
                                    // Simple click - toggle pause/play
                                    timer_commands.write(TimerCommand::Toggle);
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

// Flip the newly (re)spawned main hourglass when a color/shape change requested
// it. Shape/color changes despawn and respawn the hourglass entity, so the flip
// must land on the fresh entity (matched by `Added<MainHourglass>` the frame after
// the recreation command flushes), never the about-to-be-despawned old one.
fn apply_pending_flip(
    mut pending: ResMut<PendingFlip>,
    mut query: Query<&mut Hourglass, (With<MainHourglass>, Added<MainHourglass>)>,
) {
    if !pending.0 {
        return;
    }
    // Clear the flag only once we actually flip, so a request made while a previous
    // flip was blocking recreation survives to the real respawn.
    if let Ok(mut hourglass) = query.single_mut() {
        if hourglass.can_flip() {
            // Mirror the drag-flip (493-498): start with all sand in the bottom so
            // the crate's end-of-flip chamber swap leaves the top full.
            hourglass.upper_chamber = 0.0;
            hourglass.lower_chamber = 1.0;
            hourglass.flip();
            pending.0 = false;
        }
    }
}

fn handle_timer_start(
    timer_state: Res<TimerState>,
    pending: Res<PendingFlip>,
    mut hourglass_query: Query<&mut Hourglass, With<MainHourglass>>,
    mut last_running_state: Local<bool>,
    mut has_ever_started: Local<bool>,
) {
    // Check if timer state changed from not running to running
    if timer_state.is_running && !*last_running_state {
        // Only flip on the very first start (when timer hasn't been started before)
        if !*has_ever_started {
            // Skip the first-start flip when a color/shape change already queued a
            // flip: that flip is owned by `apply_pending_flip` on the recreated
            // entity. Flipping the old entity here would set `flipping=true` and trip
            // the recreation guard, dropping the shape/color change.
            if !pending.0 {
                for mut hourglass in hourglass_query.iter_mut() {
                    if hourglass.can_flip() {
                        hourglass.flip();
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    // --- lerp_f32 ---------------------------------------------------------

    #[test]
    fn lerp_endpoints_and_midpoint() {
        assert_abs_diff_eq!(lerp_f32(0.0, 10.0, 0.0), 0.0, epsilon = 1e-6);
        assert_abs_diff_eq!(lerp_f32(0.0, 10.0, 1.0), 10.0, epsilon = 1e-6);
        assert_abs_diff_eq!(lerp_f32(0.0, 10.0, 0.5), 5.0, epsilon = 1e-6);
        assert_abs_diff_eq!(lerp_f32(10.0, 0.0, 0.25), 7.5, epsilon = 1e-6);
    }

    #[test]
    fn lerp_extrapolates_outside_unit_interval() {
        // No clamping: t outside [0, 1] extrapolates.
        assert_abs_diff_eq!(lerp_f32(0.0, 10.0, 2.0), 20.0, epsilon = 1e-6);
    }

    // --- interpolate_bulb_style -------------------------------------------

    /// Destructure a `Circular` bulb or panic.
    fn circular(style: &BulbStyle) -> (f32, f32, usize) {
        match style {
            BulbStyle::Circular {
                curvature,
                width_factor,
                curve_resolution,
            } => (*curvature, *width_factor, *curve_resolution),
            other => panic!("expected Circular, got {other:?}"),
        }
    }

    #[test]
    fn interpolate_bulb_circular_midpoint() {
        // Classic (1.0, 1.0, 20) <-> Slim (1.5, 0.7, 18) at t = 0.5.
        let a = BulbStyle::Circular {
            curvature: 1.0,
            width_factor: 1.0,
            curve_resolution: 20,
        };
        let b = BulbStyle::Circular {
            curvature: 1.5,
            width_factor: 0.7,
            curve_resolution: 18,
        };
        let (curv, wf, res) = circular(&interpolate_bulb_style(&a, &b, 0.5));
        assert_abs_diff_eq!(curv, 1.25, epsilon = 1e-6);
        assert_abs_diff_eq!(wf, 0.85, epsilon = 1e-6);
        // lerp(20, 18, 0.5) = 19.0 -> as usize -> 19, .max(5) -> 19.
        assert_eq!(res, 19);
    }

    #[test]
    fn interpolate_bulb_curve_resolution_floor() {
        // Both low-resolution: lerp stays below 5, so the .max(5) floor applies.
        let a = BulbStyle::Circular {
            curvature: 1.0,
            width_factor: 1.0,
            curve_resolution: 2,
        };
        let b = BulbStyle::Circular {
            curvature: 1.0,
            width_factor: 1.0,
            curve_resolution: 4,
        };
        let (_, _, res) = circular(&interpolate_bulb_style(&a, &b, 0.5));
        assert_eq!(res, 5);
    }

    #[test]
    fn interpolate_bulb_mixed_variants_switch_at_half() {
        // Circular <-> Straight are different variants: clone style1 below 0.5,
        // style2 at/above 0.5.
        let circ = BulbStyle::Circular {
            curvature: 1.0,
            width_factor: 1.0,
            curve_resolution: 20,
        };
        let straight = BulbStyle::Straight { width_factor: 0.5 };
        assert!(matches!(
            interpolate_bulb_style(&circ, &straight, 0.4),
            BulbStyle::Circular { .. }
        ));
        assert!(matches!(
            interpolate_bulb_style(&circ, &straight, 0.5),
            BulbStyle::Straight { .. }
        ));
    }

    // --- interpolate_neck_style -------------------------------------------

    /// Destructure a `Curved` neck or panic.
    fn curved(style: &NeckStyle) -> (f32, f32, f32, usize) {
        match style {
            NeckStyle::Curved {
                curvature,
                width,
                height,
                curve_resolution,
            } => (*curvature, *width, *height, *curve_resolution),
            other => panic!("expected Curved, got {other:?}"),
        }
    }

    /// Destructure a `Straight` neck or panic.
    fn straight(style: &NeckStyle) -> (f32, f32) {
        match style {
            NeckStyle::Straight { width, height } => (*width, *height),
            other => panic!("expected Straight, got {other:?}"),
        }
    }

    #[test]
    fn interpolate_neck_curved_midpoint() {
        // Classic (1.0, 14, 20, 10) <-> Slim (1.5, 12, 24, 8) at t = 0.5.
        let a = NeckStyle::Curved {
            curvature: 1.0,
            width: 14.0,
            height: 20.0,
            curve_resolution: 10,
        };
        let b = NeckStyle::Curved {
            curvature: 1.5,
            width: 12.0,
            height: 24.0,
            curve_resolution: 8,
        };
        let (curv, w, h, res) = curved(&interpolate_neck_style(&a, &b, 0.5));
        assert_abs_diff_eq!(curv, 1.25, epsilon = 1e-6);
        assert_abs_diff_eq!(w, 13.0, epsilon = 1e-6);
        assert_abs_diff_eq!(h, 22.0, epsilon = 1e-6);
        // lerp(10, 8, 0.5) = 9.0 -> 9, .max(3) -> 9.
        assert_eq!(res, 9);
    }

    #[test]
    fn interpolate_neck_straight_midpoint() {
        let a = NeckStyle::Straight {
            width: 12.0,
            height: 32.0,
        };
        let b = NeckStyle::Straight {
            width: 12.0,
            height: 32.0,
        };
        let (w, h) = straight(&interpolate_neck_style(&a, &b, 0.5));
        assert_abs_diff_eq!(w, 12.0, epsilon = 1e-6);
        assert_abs_diff_eq!(h, 32.0, epsilon = 1e-6);
    }

    #[test]
    fn interpolate_neck_straight_to_curved_curvature_ramps_from_zero() {
        let s = NeckStyle::Straight {
            width: 12.0,
            height: 32.0,
        };
        let c = NeckStyle::Curved {
            curvature: 1.5,
            width: 12.0,
            height: 24.0,
            curve_resolution: 8,
        };
        // At t = 0 curvature starts at 0; resolution comes from the curved end.
        let (curv0, _, _, res0) = curved(&interpolate_neck_style(&s, &c, 0.0));
        assert_abs_diff_eq!(curv0, 0.0, epsilon = 1e-6);
        assert_eq!(res0, 8);
        // At t = 1 curvature reaches the target.
        let (curv1, _, _, _) = curved(&interpolate_neck_style(&s, &c, 1.0));
        assert_abs_diff_eq!(curv1, 1.5, epsilon = 1e-6);
    }

    #[test]
    fn interpolate_neck_curved_to_straight_curvature_ramps_to_zero() {
        let c = NeckStyle::Curved {
            curvature: 1.0,
            width: 14.0,
            height: 20.0,
            curve_resolution: 10,
        };
        let s = NeckStyle::Straight {
            width: 12.0,
            height: 32.0,
        };
        // Result stays Curved, resolution from the curved (style1) end.
        let (curv0, _, _, res0) = curved(&interpolate_neck_style(&c, &s, 0.0));
        assert_abs_diff_eq!(curv0, 1.0, epsilon = 1e-6);
        assert_eq!(res0, 10);
        let (curv1, _, _, _) = curved(&interpolate_neck_style(&c, &s, 1.0));
        assert_abs_diff_eq!(curv1, 0.0, epsilon = 1e-6);
    }

    // --- get_morphed_shape_config -----------------------------------------

    #[test]
    fn morph_anchor_t0_is_classic() {
        let (body, plates) = get_morphed_shape_config(0.0);
        let (classic_body, classic_plates) = get_main_shape_config(HourglassShape::Classic);
        assert_abs_diff_eq!(body.total_height, classic_body.total_height, epsilon = 1e-4);
        assert_abs_diff_eq!(plates.width, classic_plates.width, epsilon = 1e-4);
        assert_abs_diff_eq!(plates.height, classic_plates.height, epsilon = 1e-4);
    }

    #[test]
    fn morph_anchor_t025_is_modern() {
        let (body, plates) = get_morphed_shape_config(0.25);
        let (modern_body, modern_plates) = get_main_shape_config(HourglassShape::Modern);
        assert_abs_diff_eq!(body.total_height, modern_body.total_height, epsilon = 1e-4);
        assert_abs_diff_eq!(plates.width, modern_plates.width, epsilon = 1e-4);
        assert_abs_diff_eq!(plates.height, modern_plates.height, epsilon = 1e-4);
    }

    #[test]
    fn morph_halfway_classic_to_modern() {
        // t = 0.125 -> segment 0.5 -> Classic<->Modern at local_t 0.5.
        let (body, plates) = get_morphed_shape_config(0.125);
        // Classic and Modern share total_height 400.0.
        assert_abs_diff_eq!(body.total_height, 400.0, epsilon = 1e-4);
        // Plates width: lerp(400, 380, 0.5) = 390; height: lerp(10, 12, 0.5) = 11.
        assert_abs_diff_eq!(plates.width, 390.0, epsilon = 1e-4);
        assert_abs_diff_eq!(plates.height, 11.0, epsilon = 1e-4);
    }

    #[test]
    fn morph_wraps_at_t1_back_to_classic() {
        // t = 1.0 -> segment 4.0 -> floor % 4 == 0 -> Classic, local_t 0.
        let (body, plates) = get_morphed_shape_config(1.0);
        let (classic_body, classic_plates) = get_main_shape_config(HourglassShape::Classic);
        assert_abs_diff_eq!(body.total_height, classic_body.total_height, epsilon = 1e-4);
        assert_abs_diff_eq!(plates.width, classic_plates.width, epsilon = 1e-4);
    }

    #[test]
    fn morph_total_height_finite_and_positive_across_sweep() {
        for i in 0..10 {
            let t = i as f32 / 10.0;
            let (body, _) = get_morphed_shape_config(t);
            assert!(
                body.total_height.is_finite() && body.total_height > 0.0,
                "t = {t}: total_height = {}",
                body.total_height
            );
        }
    }

    // --- within_click_radius ----------------------------------------------

    #[test]
    fn within_click_radius_boundary_is_exclusive() {
        let center = Vec2::ZERO;
        // Dead center and just inside are hits.
        assert!(within_click_radius(center, center, 30.0, 1.0));
        assert!(within_click_radius(Vec2::new(29.0, 0.0), center, 30.0, 1.0));
        // Exactly on the boundary is a miss (strict `<`), as is beyond it.
        assert!(!within_click_radius(
            Vec2::new(30.0, 0.0),
            center,
            30.0,
            1.0
        ));
        assert!(!within_click_radius(
            Vec2::new(31.0, 0.0),
            center,
            30.0,
            1.0
        ));
    }

    #[test]
    fn within_click_radius_scale_grows_and_shrinks_hit_area() {
        let center = Vec2::ZERO;
        let point = Vec2::new(35.0, 0.0);
        // At base radius 30 the point is outside, but a hovered button (scale
        // 1.3 -> effective radius 39) pulls it inside.
        assert!(!within_click_radius(point, center, 30.0, 1.0));
        assert!(within_click_radius(point, center, 30.0, 1.3));
        // A shrunk button (scale 0.5 -> radius 15) excludes a point the base
        // radius would have included.
        let near = Vec2::new(20.0, 0.0);
        assert!(within_click_radius(near, center, 30.0, 1.0));
        assert!(!within_click_radius(near, center, 30.0, 0.5));
    }

    #[test]
    fn within_click_radius_handles_offset_center_and_smaller_radius() {
        let center = Vec2::new(100.0, 50.0);
        // 10px away with the 20px button radius -> hit.
        assert!(within_click_radius(
            Vec2::new(100.0, 60.0),
            center,
            20.0,
            1.0
        ));
        // Exactly 20px away -> miss.
        assert!(!within_click_radius(
            Vec2::new(120.0, 50.0),
            center,
            20.0,
            1.0
        ));
    }

    #[test]
    #[cfg(feature = "chrome_extension")]
    fn sidebar_scale_responds_to_width_and_open_sections() {
        let collapsed = sidebar_hourglass_scale(360.0, 800.0, false, false);
        let appearance_open = sidebar_hourglass_scale(360.0, 800.0, true, false);
        let narrow = sidebar_hourglass_scale(260.0, 800.0, false, false);
        assert!(appearance_open <= collapsed);
        assert!(narrow < collapsed);
        assert!((0.35..=1.0).contains(&collapsed));
    }

    #[test]
    #[cfg(feature = "chrome_extension")]
    fn sidebar_hit_radius_tracks_render_scale() {
        assert_abs_diff_eq!(main_hourglass_hit_radius(1.0), 220.0, epsilon = 1e-6);
        assert_abs_diff_eq!(main_hourglass_hit_radius(0.5), 110.0, epsilon = 1e-6);
    }

    // --- exceeds_drag_threshold -------------------------------------------

    #[test]
    fn exceeds_drag_threshold_boundary_is_exclusive() {
        let start = Vec2::new(5.0, 5.0);
        // Below the threshold stays a click.
        assert!(!exceeds_drag_threshold(start, Vec2::new(11.0, 5.0), 10.0));
        // Exactly at the threshold is still a click (strict `>`).
        assert!(!exceeds_drag_threshold(start, Vec2::new(15.0, 5.0), 10.0));
        // Beyond the threshold becomes a drag.
        assert!(exceeds_drag_threshold(start, Vec2::new(16.0, 5.0), 10.0));
    }

    #[test]
    fn exceeds_drag_threshold_zero_movement_is_click() {
        let p = Vec2::new(42.0, 7.0);
        assert!(!exceeds_drag_threshold(p, p, 10.0));
    }

    // --- apply_pending_flip -----------------------------------------------
    //
    // These exercise the flip-on-change orchestration with a headless `App`.
    // A `MainHourglass` is spawned via `Commands` in `Startup` so it becomes
    // visible to `apply_pending_flip`'s `Added<MainHourglass>` filter only
    // after the command buffer flushes — mirroring how the recreation systems
    // respawn the entity one frame before the flip lands on it.

    /// Build a one-tick app that spawns a default `MainHourglass` via commands
    /// and runs `apply_pending_flip` once in `Update`.
    fn flip_test_app(pending: bool) -> App {
        let mut app = App::new();
        app.init_resource::<PendingFlip>();
        app.world_mut().resource_mut::<PendingFlip>().0 = pending;
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((MainHourglass, Hourglass::default()));
        });
        app.add_systems(Update, apply_pending_flip);
        app.update();
        app
    }

    fn single_main_hourglass(app: &mut App) -> Hourglass {
        let mut query = app
            .world_mut()
            .query_filtered::<&Hourglass, With<MainHourglass>>();
        query.single(app.world()).unwrap().clone()
    }

    #[test]
    fn pending_flip_flips_the_recreated_hourglass() {
        let mut app = flip_test_app(true);

        let hourglass = single_main_hourglass(&mut app);
        assert!(hourglass.flipping, "a pending flip should start the flip");
        // Sand forced to the bottom so the crate's end-of-flip swap leaves the
        // top full, matching the drag-flip path.
        assert_eq!(hourglass.upper_chamber, 0.0);
        assert_eq!(hourglass.lower_chamber, 1.0);
        // The request is consumed exactly once.
        assert!(!app.world().resource::<PendingFlip>().0);
    }

    #[test]
    fn no_pending_flip_leaves_the_hourglass_untouched() {
        let mut app = flip_test_app(false);

        let hourglass = single_main_hourglass(&mut app);
        assert!(
            !hourglass.flipping,
            "without a pending flip the hourglass must not flip on spawn"
        );
        assert!(!app.world().resource::<PendingFlip>().0);
    }

    // --- handle_timer_start -----------------------------------------------
    //
    // The `MainHourglass` is spawned in `Startup` so it is present before
    // `handle_timer_start` runs in `Update` on the same tick. Both `Local`s
    // start `false`, so a running timer reads as "just transitioned to running"
    // — the first-start condition.

    /// One-tick app: a default `MainHourglass`, the given `TimerState`, and a
    /// `PendingFlip`, with `handle_timer_start` in `Update`.
    fn timer_start_app(timer_state: TimerState, pending: bool) -> App {
        let mut app = App::new();
        app.insert_resource(timer_state);
        app.init_resource::<PendingFlip>();
        app.world_mut().resource_mut::<PendingFlip>().0 = pending;
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((MainHourglass, Hourglass::default()));
        });
        app.add_systems(Update, handle_timer_start);
        app.update();
        app
    }

    #[test]
    fn first_start_flips_when_no_pending() {
        let mut app = timer_start_app(
            TimerState {
                duration: 100.0,
                remaining: 50.0,
                is_running: true,
            },
            false,
        );
        let hourglass = single_main_hourglass(&mut app);
        assert!(
            hourglass.flipping,
            "the very first start should flip the hourglass"
        );
    }

    #[test]
    fn first_start_skips_flip_when_pending() {
        // A queued color/shape flip owns the animation on the recreated entity,
        // so handle_timer_start must not flip the current one.
        let mut app = timer_start_app(
            TimerState {
                duration: 100.0,
                remaining: 50.0,
                is_running: true,
            },
            true,
        );
        let hourglass = single_main_hourglass(&mut app);
        assert!(
            !hourglass.flipping,
            "a pending flip must suppress the first-start flip"
        );
    }

    #[test]
    fn at_rest_timer_does_not_flip() {
        // remaining == duration && !running drives the has_ever_started reset
        // branch; nothing should flip.
        let mut app = timer_start_app(
            TimerState {
                duration: 100.0,
                remaining: 100.0,
                is_running: false,
            },
            false,
        );
        let hourglass = single_main_hourglass(&mut app);
        assert!(!hourglass.flipping, "an at-rest timer must not flip");
    }

    // --- update_hourglass_timer -------------------------------------------

    /// One-tick app: a default `MainHourglass` and the given `TimerState`, with
    /// `update_hourglass_timer` in `Update`.
    fn timer_sync_app(timer_state: TimerState) -> App {
        let mut app = App::new();
        app.insert_resource(timer_state);
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((MainHourglass, Hourglass::default()));
        });
        app.add_systems(Update, update_hourglass_timer);
        app.update();
        app
    }

    #[test]
    fn update_hourglass_timer_syncs_state_and_chambers() {
        let mut app = timer_sync_app(TimerState {
            duration: 100.0,
            remaining: 25.0,
            is_running: true,
        });
        let hourglass = single_main_hourglass(&mut app);
        assert_abs_diff_eq!(hourglass.total_time, 100.0, epsilon = 1e-6);
        assert_abs_diff_eq!(hourglass.remaining_time, 25.0, epsilon = 1e-6);
        assert!(hourglass.running);
        // Chambers track progress: 25% remaining on top, 75% elapsed below.
        assert_abs_diff_eq!(hourglass.upper_chamber, 0.25, epsilon = 1e-6);
        assert_abs_diff_eq!(hourglass.lower_chamber, 0.75, epsilon = 1e-6);
    }

    #[test]
    fn update_hourglass_timer_zero_duration_leaves_chambers_default() {
        // With duration 0 the chamber branch is skipped, so the defaults survive
        // (a default Hourglass starts full on top).
        let mut app = timer_sync_app(TimerState {
            duration: 0.0,
            remaining: 0.0,
            is_running: false,
        });
        let default_hg = Hourglass::default();
        let hourglass = single_main_hourglass(&mut app);
        assert_abs_diff_eq!(
            hourglass.upper_chamber,
            default_hg.upper_chamber,
            epsilon = 1e-6
        );
        assert_abs_diff_eq!(
            hourglass.lower_chamber,
            default_hg.lower_chamber,
            epsilon = 1e-6
        );
        assert!(!hourglass.running);
    }

    // --- update_hourglass_color -------------------------------------------

    #[test]
    fn update_hourglass_color_applies_config_color() {
        let mut app = App::new();
        app.init_resource::<HourglassConfig>();
        let new_color = Color::srgb(0.1, 0.2, 0.3);
        app.world_mut().resource_mut::<HourglassConfig>().color = new_color;
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((MainHourglass, Hourglass::default()));
        });
        app.add_systems(Update, update_hourglass_color);
        app.update();

        let hourglass = single_main_hourglass(&mut app);
        assert_eq!(hourglass.sand_color, new_color);
    }
}
