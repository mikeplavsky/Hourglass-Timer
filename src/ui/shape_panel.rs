use crate::resources::{HourglassConfig, HourglassShape, ShapeMode};
use crate::ui::ShapeRowMarker;
use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy_hourglass::{Hourglass, HourglassMeshBuilder, HourglassMeshSandConfig};
use rand::Rng;

use crate::hourglass::get_mini_shape_config;

// Bevy's default font is an ASCII-only FiraMono subset. We embed Fira Sans
// Regular into the binary so the shape-row buttons can render non-ASCII
// glyphs (e.g. ∞) without needing a sibling `assets/` directory at runtime.
const SHAPE_BUTTON_FONT: &str = "embedded://hourglass_timer/ui/fonts/FiraSans-Regular.ttf";

/// Shared font handle for the 3D shape-row buttons. Loaded once at startup so we
/// don't kick off duplicate async loads from each spawn system.
#[derive(Resource)]
struct ShapeButtonFont(Handle<Font>);

pub struct ShapePanelPlugin;

impl Plugin for ShapePanelPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "fonts/FiraSans-Regular.ttf");

        app.add_systems(PreStartup, load_shape_button_font)
            .add_systems(
                PostStartup,
                (
                    spawn_shape_buttons,
                    spawn_random_shape_button,
                    spawn_morphing_button,
                ),
            )
            .add_systems(
                Update,
                (
                    update_mini_hourglass_colors,
                    handle_hover_effects,
                    update_hourglass_layering,
                    handle_shape_button_clicks.after(update_hourglass_layering),
                    handle_random_shape_button_clicks.after(update_hourglass_layering),
                    handle_morphing_button_clicks.after(update_hourglass_layering),
                    update_hover_timers,
                    update_mini_hourglass_positions,
                ),
            );
    }
}

fn load_shape_button_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ShapeButtonFont(asset_server.load(SHAPE_BUTTON_FONT)));
}

fn handle_hover_effects(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mini_hourglass_query: Query<(Entity, &Transform, &ShapeButton), With<MiniHourglass>>,
    morphing_button_query: Query<(Entity, &Transform), (With<MorphingButton>, With<MiniHourglass>)>,
    random_shape_button_query: Query<
        (Entity, &Transform),
        (With<RandomShapeButton>, With<MiniHourglass>),
    >,
    hovered_query: Query<Entity, With<HoveredHourglass>>,
) {
    if let Ok(window) = windows.single() {
        if let Some(cursor_position) = window.cursor_position() {
            if let Ok((camera, camera_transform)) = camera_query.single() {
                if let Ok(world_position) =
                    camera.viewport_to_world_2d(camera_transform, cursor_position)
                {
                    let mut currently_hovered = None;

                    // Check if hovering over any mini hourglass
                    for (entity, transform, _shape_button) in mini_hourglass_query.iter() {
                        let distance = world_position.distance(transform.translation.truncate());

                        // Adjust detection radius based on current scale
                        let detection_radius = 30.0 * transform.scale.x;

                        if distance < detection_radius {
                            currently_hovered = Some(entity);
                            break;
                        }
                    }

                    // Check if hovering over the random shape button
                    if currently_hovered.is_none() {
                        if let Ok((entity, transform)) = random_shape_button_query.single() {
                            let distance =
                                world_position.distance(transform.translation.truncate());
                            let detection_radius = 20.0 * transform.scale.x;

                            if distance < detection_radius {
                                currently_hovered = Some(entity);
                            }
                        }
                    }

                    // Check if hovering over the morphing button
                    if currently_hovered.is_none() {
                        if let Ok((entity, transform)) = morphing_button_query.single() {
                            let distance =
                                world_position.distance(transform.translation.truncate());
                            let detection_radius = 20.0 * transform.scale.x;

                            if distance < detection_radius {
                                currently_hovered = Some(entity);
                            }
                        }
                    }

                    // Remove HoveredHourglass from all entities that are no longer hovered
                    for hovered_entity in hovered_query.iter() {
                        if Some(hovered_entity) != currently_hovered {
                            commands.entity(hovered_entity).remove::<HoveredHourglass>();
                        }
                    }

                    // Add HoveredHourglass to currently hovered entity if it doesn't have it
                    if let Some(hovered_entity) = currently_hovered {
                        if !hovered_query.contains(hovered_entity) {
                            commands
                                .entity(hovered_entity)
                                .insert(HoveredHourglass { timer: 0.0 });
                        }
                    }
                }
            }
        }
    }
}

fn update_hourglass_layering(
    config: Res<HourglassConfig>,
    mut mini_hourglass_query: Query<(
        &mut Transform,
        &MiniHourglass,
        &ShapeButton,
        Option<&HoveredHourglass>,
    )>,
    mut morphing_button_query: Query<
        (&mut Transform, &MiniHourglass, Option<&HoveredHourglass>),
        (With<MorphingButton>, Without<ShapeButton>),
    >,
    mut random_shape_button_query: Query<
        (&mut Transform, &MiniHourglass, Option<&HoveredHourglass>),
        (
            With<RandomShapeButton>,
            Without<ShapeButton>,
            Without<MorphingButton>,
        ),
    >,
) {
    // Handle regular hourglass buttons
    for (mut transform, mini_hourglass, shape_button, hovered) in mini_hourglass_query.iter_mut() {
        let base_position = mini_hourglass.base_position;

        // Visual effects with scaling only
        let scale = if let Some(_hover_component) = hovered {
            // Hovered state: larger scale
            1.3
        } else if config.shape_type == shape_button.shape {
            // Selected state: slightly larger
            1.15
        } else {
            // Default state
            1.0
        };

        // Apply scale
        transform.scale = Vec3::splat(scale);

        // Keep original position
        transform.translation = base_position;
    }

    // Handle morphing button
    if let Ok((mut transform, mini_hourglass, hovered)) = morphing_button_query.single_mut() {
        let base_position = mini_hourglass.base_position;

        // Visual effects with scaling only
        let scale = if let Some(_hover_component) = hovered {
            // Hovered state: larger scale
            1.3
        } else if config.shape_mode == ShapeMode::Morphing {
            // Selected state: slightly larger when morphing is active
            1.15
        } else {
            // Default state
            1.0
        };

        // Apply scale
        transform.scale = Vec3::splat(scale);

        // Keep original position
        transform.translation = base_position;
    }

    // Handle random shape button (no persistent selected state — momentary action)
    if let Ok((mut transform, mini_hourglass, hovered)) = random_shape_button_query.single_mut() {
        let base_position = mini_hourglass.base_position;

        let scale = if hovered.is_some() { 1.3 } else { 1.0 };

        transform.scale = Vec3::splat(scale);
        transform.translation = base_position;
    }
}

fn update_hover_timers(time: Res<Time>, mut hovered_query: Query<&mut HoveredHourglass>) {
    for mut hovered in hovered_query.iter_mut() {
        hovered.timer += time.delta_secs();

        // Optional: Remove hover effect after some time if desired
        // For now, we'll keep it until the mouse moves away
    }
}

fn update_mini_hourglass_colors(
    config: Res<HourglassConfig>,
    mut query: Query<&mut bevy_hourglass::HourglassMeshSandState, With<MiniHourglass>>,
) {
    if config.is_changed() {
        for mut sand_state in query.iter_mut() {
            sand_state.sand_config.color = config.color;
            sand_state.needs_update = true;
        }
    }
}

fn update_mini_hourglass_positions(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    shape_row_query: Query<&Node, With<ShapeRowMarker>>,
    mut mini_hourglass_query: Query<(&mut Transform, &mut MiniHourglass), With<MiniHourglass>>,
) {
    // Check if the shape row UI node exists to ensure proper positioning
    if shape_row_query.single().is_ok() {
        if let Ok(window) = windows.single() {
            if let Ok((camera, camera_transform)) = camera_query.single() {
                // Calculate the center of the shape row area
                let window_width = window.width();

                // Calculate shape row position based on UI layout:
                let shape_row_center_y = 60.0;

                // Convert screen space to world space for the shape row center
                let shape_row_screen_pos = Vec2::new(window_width / 2.0, shape_row_center_y);

                if let Ok(shape_row_world_pos) =
                    camera.viewport_to_world_2d(camera_transform, shape_row_screen_pos)
                {
                    // Update each mini hourglass position relative to the shape row
                    for (mut transform, mut mini_hourglass) in mini_hourglass_query.iter_mut() {
                        // Calculate new position based on original X offset from center
                        let new_position = Vec3::new(
                            shape_row_world_pos.x + mini_hourglass.original_x,
                            shape_row_world_pos.y,
                            10.0, // Keep elevated Z position
                        );

                        // Update both current transform and stored base position
                        transform.translation = new_position;
                        mini_hourglass.base_position = new_position;
                    }
                }
            }
        }
    }
}

#[derive(Component)]
struct ShapeButton {
    shape: HourglassShape,
}

#[derive(Component)]
struct MorphingButton;

#[derive(Component)]
struct RandomShapeButton;

#[derive(Component)]
struct MiniHourglass {
    base_position: Vec3, // Store the original position
    original_x: f32,     // Store the original X position for positioning
}

#[derive(Component)]
struct HoveredHourglass {
    timer: f32, // Timer for hover effect duration
}

fn spawn_shape_buttons(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<HourglassConfig>,
) {
    // Spawn mini hourglasses in 3D space positioned horizontally for the shape row
    let shapes = [
        HourglassShape::Classic,
        HourglassShape::Modern,
        HourglassShape::Slim,
        HourglassShape::Wide,
    ];

    for (i, shape) in shapes.iter().enumerate() {
        // Calculate offset from center for horizontal spacing
        let x_offset = -100.0 + (i as f32 * 50.0); // Offset from center

        let (body_config, plates_config) = get_mini_shape_config(*shape);

        // Start with a temporary position - will be updated by update_mini_hourglass_positions
        let temp_position = Vec3::new(0.0, 0.0, 10.0);

        let entity = HourglassMeshBuilder::new(Transform::from_translation(temp_position))
            .with_body(body_config)
            .with_plates(plates_config)
            .with_sand(HourglassMeshSandConfig {
                color: config.color,
                fill_percent: 0.7, // Partially filled for visual appeal
                wall_offset: 1.0,
            })
            .build(&mut commands, &mut meshes, &mut materials);

        // Remove the Hourglass component from mini hourglasses since they should be static displays
        commands.entity(entity).remove::<Hourglass>();

        commands.entity(entity).insert((
            MiniHourglass {
                base_position: temp_position,
                original_x: x_offset, // Store the offset from center
            },
            ShapeButton { shape: *shape }, // Make it clickable
            Name::new(format!("Mini Hourglass {shape:?}")),
        ));
    }
}

fn spawn_morphing_button(mut commands: Commands, font: Res<ShapeButtonFont>) {
    let x_offset = 150.0;
    let temp_position = Vec3::new(0.0, 0.0, 10.0);

    let button_entity = commands
        .spawn((
            Name::new("Morphing Button 3D"),
            MorphingButton,
            Transform::from_translation(temp_position),
            Visibility::default(),
            MiniHourglass {
                base_position: temp_position,
                original_x: x_offset,
            },
        ))
        .id();

    commands.entity(button_entity).with_children(|parent| {
        parent.spawn((
            Name::new("Infinity Text"),
            Text2d::new("∞"),
            TextColor(Color::WHITE),
            TextFont {
                font: font.0.clone(),
                font_size: 32.0,
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
        ));
    });
}

fn spawn_random_shape_button(mut commands: Commands, font: Res<ShapeButtonFont>) {
    let x_offset = 100.0;
    let temp_position = Vec3::new(0.0, 0.0, 10.0);

    let button_entity = commands
        .spawn((
            Name::new("Random Shape Button 3D"),
            RandomShapeButton,
            Transform::from_translation(temp_position),
            Visibility::default(),
            MiniHourglass {
                base_position: temp_position,
                original_x: x_offset,
            },
        ))
        .id();

    commands.entity(button_entity).with_children(|parent| {
        parent.spawn((
            Name::new("Random Shape Text"),
            Text2d::new("?"),
            TextColor(Color::WHITE),
            TextFont {
                font: font.0.clone(),
                font_size: 32.0,
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
        ));
    });
}

fn handle_random_shape_button_clicks(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    random_shape_button_query: Query<&Transform, (With<RandomShapeButton>, With<MiniHourglass>)>,
    mut config: ResMut<HourglassConfig>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Ok(window) = windows.single() {
            if let Some(cursor_position) = window.cursor_position() {
                if let Ok((camera, camera_transform)) = camera_query.single() {
                    if let Ok(world_position) =
                        camera.viewport_to_world_2d(camera_transform, cursor_position)
                    {
                        if let Ok(transform) = random_shape_button_query.single() {
                            let distance =
                                world_position.distance(transform.translation.truncate());

                            let click_radius = 20.0 * transform.scale.x;

                            if distance < click_radius {
                                let shapes = [
                                    HourglassShape::Classic,
                                    HourglassShape::Modern,
                                    HourglassShape::Slim,
                                    HourglassShape::Wide,
                                ];
                                let mut rng = rand::thread_rng();
                                config.shape_type = shapes[rng.gen_range(0..shapes.len())];
                                config.shape_mode = ShapeMode::Static;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn handle_morphing_button_clicks(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    morphing_button_query: Query<&Transform, (With<MorphingButton>, With<MiniHourglass>)>,
    mut config: ResMut<HourglassConfig>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Ok(window) = windows.single() {
            if let Some(cursor_position) = window.cursor_position() {
                if let Ok((camera, camera_transform)) = camera_query.single() {
                    // Convert screen coordinates to world coordinates
                    if let Ok(world_position) =
                        camera.viewport_to_world_2d(camera_transform, cursor_position)
                    {
                        // Check if click is near the morphing button
                        if let Ok(transform) = morphing_button_query.single() {
                            let distance =
                                world_position.distance(transform.translation.truncate());

                            // Adjust click detection radius based on current scale
                            let click_radius = 20.0 * transform.scale.x;

                            if distance < click_radius {
                                // Toggle morphing mode
                                if config.shape_mode == ShapeMode::Static {
                                    config.shape_mode = ShapeMode::Morphing;
                                } else {
                                    config.shape_mode = ShapeMode::Static;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn handle_shape_button_clicks(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mini_hourglass_query: Query<(&Transform, &ShapeButton), With<MiniHourglass>>,
    mut config: ResMut<HourglassConfig>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Ok(window) = windows.single() {
            if let Some(cursor_position) = window.cursor_position() {
                if let Ok((camera, camera_transform)) = camera_query.single() {
                    // Convert screen coordinates to world coordinates
                    if let Ok(world_position) =
                        camera.viewport_to_world_2d(camera_transform, cursor_position)
                    {
                        // Check if click is near any mini hourglass
                        for (transform, shape_button) in mini_hourglass_query.iter() {
                            let distance =
                                world_position.distance(transform.translation.truncate());

                            // Adjust click detection radius based on current scale
                            let click_radius = 30.0 * transform.scale.x;

                            if distance < click_radius {
                                config.shape_type = shape_button.shape;
                                config.shape_mode = ShapeMode::Static; // Set to static when selecting a specific shape
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}
