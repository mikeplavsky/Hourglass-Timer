use bevy::prelude::*;
use bevy_hourglass::{Hourglass, HourglassMeshBuilder, HourglassMeshSandConfig};
use crate::resources::{HourglassConfig, HourglassShape};

use crate::hourglass::get_mini_shape_config;

pub struct ShapePanelPlugin;

impl Plugin for ShapePanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_shape_buttons)
            .add_systems(Update, (
                handle_shape_button_clicks,
                update_mini_hourglass_colors,
                handle_hover_effects,
                update_hourglass_layering,
                update_hover_timers,
            ));
    }
}

fn handle_hover_effects(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mini_hourglass_query: Query<(Entity, &Transform, &ShapeButton), With<MiniHourglass>>,
    hovered_query: Query<Entity, With<HoveredHourglass>>,
) {
    if let Ok(window) = windows.single() {
        if let Some(cursor_position) = window.cursor_position() {
            if let Ok((camera, camera_transform)) = camera_query.single() {
                if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
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

                    // Remove HoveredHourglass from all entities that are no longer hovered
                    for hovered_entity in hovered_query.iter() {
                        if Some(hovered_entity) != currently_hovered {
                            commands.entity(hovered_entity).remove::<HoveredHourglass>();
                        }
                    }

                    // Add HoveredHourglass to currently hovered entity if it doesn't have it
                    if let Some(hovered_entity) = currently_hovered {
                        if !hovered_query.contains(hovered_entity) {
                            commands.entity(hovered_entity).insert(HoveredHourglass { timer: 0.0 });
                        }
                    }
                }
            }
        }
    }
}

fn update_hourglass_layering(
    config: Res<HourglassConfig>,
    time: Res<Time>,
    mut mini_hourglass_query: Query<(&mut Transform, &MiniHourglass, &ShapeButton, Option<&HoveredHourglass>)>,
) {
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
}

fn update_hover_timers(
    time: Res<Time>,
    mut hovered_query: Query<&mut HoveredHourglass>,
) {
    for mut hovered in hovered_query.iter_mut() {
        hovered.timer += time.delta_secs();

        // Optional: Remove hover effect after some time if desired
        // For now, we'll keep it until the mouse moves away
    }
}

fn update_mini_hourglass_colors(
    config: Res<HourglassConfig>,
    mut query: Query<&mut Hourglass, With<MiniHourglass>>,
) {
    if config.is_changed() {
        for mut hourglass in query.iter_mut() {
            hourglass.sand_color = config.color;
        }
    }
}



#[derive(Component)]
struct ShapeButton {
    shape: HourglassShape,
}

#[derive(Component)]
struct MiniHourglass {
    base_position: Vec3, // Store the original position
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
        // Position them horizontally across the top area
        let x_pos = -200.0 + (i as f32 * 120.0); // Spaced horizontally
        let y_pos = 200.0; // Position in the top area for shape row

        let (body_config, plates_config) = get_mini_shape_config(*shape);

        let base_z = 10.0; // Start with elevated base position
        let position = Vec3::new(x_pos, y_pos, base_z);

        let entity = HourglassMeshBuilder::new(Transform::from_translation(position))
            .with_body(body_config)
            .with_plates(plates_config)
            .with_sand(HourglassMeshSandConfig {
                color: config.color,
                fill_percent: 0.7, // Partially filled for visual appeal
                wall_offset: 1.0,
            })
            .build(&mut commands, &mut meshes, &mut materials);

        commands.entity(entity).insert((
            MiniHourglass {
                base_position: position,
            },
            ShapeButton { shape: *shape }, // Make it clickable
            Name::new(format!("Mini Hourglass {:?}", shape)),
        ));
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
                    if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
                        // Check if click is near any mini hourglass
                        for (transform, shape_button) in mini_hourglass_query.iter() {
                            let distance = world_position.distance(transform.translation.truncate());

                            // Adjust click detection radius based on current scale
                            let click_radius = 30.0 * transform.scale.x;

                            if distance < click_radius {
                                config.shape_type = shape_button.shape;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}
