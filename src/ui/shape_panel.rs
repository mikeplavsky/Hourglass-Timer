use crate::resources::{
    AppearanceStateChanged, HourglassConfig, HourglassShape, PendingFlip, SAND_COLOR, ShapeMode,
};
use crate::timer::{TimerCommand, TimerSystems};
use crate::ui::{AppearancePanelVisible, ShapeRowMarker, extension_appearance_change_command};
use bevy::asset::embedded_asset;
use bevy::prelude::*;
use bevy_hourglass::{Hourglass, HourglassMeshBuilder, HourglassMeshSandConfig};
use rand::Rng;

use crate::hourglass::{get_mini_shape_config, within_click_radius};

// Bevy's default font is an ASCII-only FiraMono subset. We embed Fira Sans
// Regular into the binary so the shape-row buttons can render non-ASCII
// glyphs (e.g. ∞) without needing a sibling `assets/` directory at runtime.
const SHAPE_BUTTON_FONT: &str = "embedded://hourglass_timer/ui/fonts/FiraSans-Regular.ttf";

pub struct ShapePanelPlugin;

impl Plugin for ShapePanelPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "fonts/FiraSans-Regular.ttf");

        app.add_systems(
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
                handle_shape_button_clicks,
                handle_random_shape_button_clicks,
                handle_morphing_button_clicks,
            )
                .in_set(TimerSystems::Input),
        )
        .add_systems(
            Update,
            (
                update_mini_hourglass_colors,
                handle_hover_effects,
                update_hourglass_layering,
                update_hover_timers,
                update_mini_hourglass_positions,
                update_shape_panel_visibility,
            ),
        );
    }
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
    appearance_visible: Res<AppearancePanelVisible>,
) {
    if !appearance_visible.0 {
        return;
    }
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

/// Scale factor for a shape-row button. Hover takes precedence over selection:
/// hovered buttons grow to 1.3, an unhovered-but-selected button sits at 1.15,
/// and everything else stays at 1.0.
fn shape_button_scale(is_hovered: bool, is_selected: bool) -> f32 {
    if is_hovered {
        1.3
    } else if is_selected {
        1.15
    } else {
        1.0
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
        let scale = shape_button_scale(hovered.is_some(), config.shape_type == shape_button.shape);

        // Apply scale
        transform.scale = Vec3::splat(scale);

        // Keep original position
        transform.translation = base_position;
    }

    // Handle morphing button
    if let Ok((mut transform, mini_hourglass, hovered)) = morphing_button_query.single_mut() {
        let base_position = mini_hourglass.base_position;

        // Visual effects with scaling only
        let scale = shape_button_scale(hovered.is_some(), config.shape_mode == ShapeMode::Morphing);

        // Apply scale
        transform.scale = Vec3::splat(scale);

        // Keep original position
        transform.translation = base_position;
    }

    // Handle random shape button (no persistent selected state — momentary action)
    if let Ok((mut transform, mini_hourglass, hovered)) = random_shape_button_query.single_mut() {
        let base_position = mini_hourglass.base_position;

        // Random button has no persistent selected state — only hover scales it.
        let scale = shape_button_scale(hovered.is_some(), false);

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
            sand_state.sand_config.color = SAND_COLOR;
            sand_state.needs_update = true;
        }
    }
}

fn update_mini_hourglass_positions(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    shape_row_query: Query<(&ComputedNode, &GlobalTransform), With<ShapeRowMarker>>,
    mut mini_hourglass_query: Query<(&mut Transform, &mut MiniHourglass), With<MiniHourglass>>,
) {
    if let Ok((shape_row_node, shape_row_transform)) = shape_row_query.single() {
        if let Ok(window) = windows.single() {
            if let Ok((camera, camera_transform)) = camera_query.single() {
                let window_width = window.width();
                let horizontal_scale = if cfg!(feature = "chrome_extension") {
                    ((window_width - 36.0) / 280.0).clamp(0.55, 1.0)
                } else {
                    1.0
                };

                #[cfg(feature = "chrome_extension")]
                let Some(shape_row_screen_pos) =
                    extension_shape_row_screen_position(shape_row_node, shape_row_transform)
                else {
                    return;
                };

                #[cfg(not(feature = "chrome_extension"))]
                let shape_row_screen_pos = {
                    let _ = (shape_row_node, shape_row_transform);
                    Vec2::new(window_width / 2.0, 60.0)
                };

                if let Ok(shape_row_world_pos) =
                    camera.viewport_to_world_2d(camera_transform, shape_row_screen_pos)
                {
                    // Update each mini hourglass position relative to the shape row
                    for (mut transform, mut mini_hourglass) in mini_hourglass_query.iter_mut() {
                        // Calculate new position based on original X offset from center
                        let new_position = Vec3::new(
                            shape_row_world_pos.x
                                + if cfg!(feature = "chrome_extension") {
                                    (mini_hourglass.original_x - 25.0) * horizontal_scale
                                } else {
                                    mini_hourglass.original_x
                                },
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

#[cfg(feature = "chrome_extension")]
fn extension_shape_row_screen_position(
    node: &ComputedNode,
    transform: &GlobalTransform,
) -> Option<Vec2> {
    (!node.is_empty()).then(|| transform.translation().truncate() * node.inverse_scale_factor())
}

fn update_shape_panel_visibility(
    appearance_visible: Res<AppearancePanelVisible>,
    mut query: Query<&mut Visibility, With<MiniHourglass>>,
) {
    if !appearance_visible.is_changed() {
        return;
    }
    for mut visibility in &mut query {
        *visibility = if appearance_visible.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
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
pub struct MiniHourglass {
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
                color: SAND_COLOR,
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

fn spawn_morphing_button(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let x_offset = 150.0;

    let temp_position = Vec3::new(0.0, 0.0, 10.0);

    let button_entity = commands
        .spawn((
            Name::new("Morphing Button 3D"),
            MorphingButton,
            Mesh2d(meshes.add(Rectangle::new(30.0, 30.0))),
            Transform::from_translation(temp_position),
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
                font: asset_server.load(SHAPE_BUTTON_FONT),
                font_size: 32.0,
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
        ));
    });
}

fn spawn_random_shape_button(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let x_offset = 100.0;

    let temp_position = Vec3::new(0.0, 0.0, 10.0);

    let button_entity = commands
        .spawn((
            Name::new("Random Shape Button 3D"),
            RandomShapeButton,
            Mesh2d(meshes.add(Rectangle::new(30.0, 30.0))),
            Transform::from_translation(temp_position),
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
                font: asset_server.load(SHAPE_BUTTON_FONT),
                font_size: 32.0,
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
        ));
    });
}

/// Pick a random hourglass shape different from `current`, re-rolling until it
/// differs.
fn pick_distinct_shape(current: HourglassShape, rng: &mut impl Rng) -> HourglassShape {
    let shapes = [
        HourglassShape::Classic,
        HourglassShape::Modern,
        HourglassShape::Slim,
        HourglassShape::Wide,
    ];
    let mut new_shape = shapes[rng.gen_range(0..shapes.len())];
    // Re-roll until we get a shape different from the current one
    while new_shape == current {
        new_shape = shapes[rng.gen_range(0..shapes.len())];
    }
    new_shape
}

fn handle_random_shape_button_clicks(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    random_shape_button_query: Query<&Transform, (With<RandomShapeButton>, With<MiniHourglass>)>,
    mut config: ResMut<HourglassConfig>,
    mut pending_flip: ResMut<PendingFlip>,
    mut timer_commands: EventWriter<TimerCommand>,
    mut appearance_changed: EventWriter<AppearanceStateChanged>,
    appearance_visible: Res<AppearancePanelVisible>,
) {
    if !appearance_visible.0 {
        return;
    }
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Ok(window) = windows.single() {
            if let Some(cursor_position) = window.cursor_position() {
                if let Ok((camera, camera_transform)) = camera_query.single() {
                    if let Ok(world_position) =
                        camera.viewport_to_world_2d(camera_transform, cursor_position)
                    {
                        if let Ok(transform) = random_shape_button_query.single() {
                            if within_click_radius(
                                world_position,
                                transform.translation.truncate(),
                                20.0,
                                transform.scale.x,
                            ) {
                                let mut rng = rand::thread_rng();
                                let new_shape = pick_distinct_shape(config.shape_type, &mut rng);
                                config.shape_type = new_shape;
                                config.shape_mode = ShapeMode::Static;
                                if let Some(command) =
                                    extension_appearance_change_command(&mut pending_flip)
                                {
                                    timer_commands.write(command);
                                }
                                appearance_changed.write_default();
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
    mut pending_flip: ResMut<PendingFlip>,
    mut timer_commands: EventWriter<TimerCommand>,
    mut appearance_changed: EventWriter<AppearanceStateChanged>,
    appearance_visible: Res<AppearancePanelVisible>,
) {
    if !appearance_visible.0 {
        return;
    }
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
                            if within_click_radius(
                                world_position,
                                transform.translation.truncate(),
                                20.0,
                                transform.scale.x,
                            ) {
                                // Toggle morphing mode
                                if config.shape_mode == ShapeMode::Static {
                                    config.shape_mode = ShapeMode::Morphing;
                                } else {
                                    config.shape_mode = ShapeMode::Static;
                                }
                                if let Some(command) =
                                    extension_appearance_change_command(&mut pending_flip)
                                {
                                    timer_commands.write(command);
                                }
                                appearance_changed.write_default();
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
    mut pending_flip: ResMut<PendingFlip>,
    mut timer_commands: EventWriter<TimerCommand>,
    mut appearance_changed: EventWriter<AppearanceStateChanged>,
    appearance_visible: Res<AppearancePanelVisible>,
) {
    if !appearance_visible.0 {
        return;
    }
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
                            if within_click_radius(
                                world_position,
                                transform.translation.truncate(),
                                30.0,
                                transform.scale.x,
                            ) {
                                config.shape_type = shape_button.shape;
                                config.shape_mode = ShapeMode::Static; // Set to static when selecting a specific shape
                                if let Some(command) =
                                    extension_appearance_change_command(&mut pending_flip)
                                {
                                    timer_commands.write(command);
                                }
                                appearance_changed.write_default();
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    const ALL_SHAPES: [HourglassShape; 4] = [
        HourglassShape::Classic,
        HourglassShape::Modern,
        HourglassShape::Slim,
        HourglassShape::Wide,
    ];

    #[test]
    fn pick_distinct_shape_always_differs_from_current() {
        for current in ALL_SHAPES {
            for seed in 0..20 {
                let mut rng = StdRng::seed_from_u64(seed);
                let new_shape = pick_distinct_shape(current, &mut rng);
                assert_ne!(new_shape, current, "current {current:?}, seed {seed}");
            }
        }
    }

    #[test]
    fn pick_distinct_shape_returns_valid_variant() {
        for current in ALL_SHAPES {
            let mut rng = StdRng::seed_from_u64(7);
            let new_shape = pick_distinct_shape(current, &mut rng);
            assert!(ALL_SHAPES.contains(&new_shape));
        }
    }

    #[test]
    fn pick_distinct_shape_is_deterministic_for_same_seed() {
        let mut rng_a = StdRng::seed_from_u64(99);
        let mut rng_b = StdRng::seed_from_u64(99);
        let a = pick_distinct_shape(HourglassShape::Classic, &mut rng_a);
        let b = pick_distinct_shape(HourglassShape::Classic, &mut rng_b);
        assert_eq!(a, b);
    }

    #[test]
    fn mini_shape_sand_ignores_selected_color() {
        let mut app = App::new();
        app.insert_resource(HourglassConfig {
            color: Color::srgb(0.1, 0.3, 0.8),
            ..default()
        });
        app.world_mut().spawn((
            MiniHourglass {
                base_position: Vec3::ZERO,
                original_x: 0.0,
            },
            bevy_hourglass::HourglassMeshSandState {
                fill_percent: 0.7,
                body_config: default(),
                sand_config: HourglassMeshSandConfig {
                    color: Color::srgb(0.1, 0.5, 0.1),
                    fill_percent: 0.7,
                    wall_offset: 1.0,
                },
                needs_update: false,
            },
        ));
        app.add_systems(Update, update_mini_hourglass_colors);

        app.update();

        let mut query = app
            .world_mut()
            .query::<&bevy_hourglass::HourglassMeshSandState>();
        let sand_state = query.single(app.world()).unwrap();
        assert_eq!(sand_state.sand_config.color, SAND_COLOR);
        assert!(sand_state.needs_update);
    }

    #[test]
    #[cfg(feature = "chrome_extension")]
    fn extension_shape_row_converts_physical_center_to_logical_viewport_position() {
        let node = ComputedNode {
            size: Vec2::new(440.0, 104.0),
            inverse_scale_factor: 0.5,
            ..default()
        };
        let transform = GlobalTransform::from_translation(Vec3::new(460.0, 164.0, 0.0));

        assert_eq!(
            extension_shape_row_screen_position(&node, &transform),
            Some(Vec2::new(230.0, 82.0))
        );
    }

    #[test]
    #[cfg(feature = "chrome_extension")]
    fn extension_shape_row_waits_for_non_empty_layout() {
        let transform = GlobalTransform::from_translation(Vec3::new(460.0, 164.0, 0.0));

        assert_eq!(
            extension_shape_row_screen_position(&ComputedNode::default(), &transform),
            None
        );
    }

    // --- shape_button_scale -----------------------------------------------

    #[test]
    fn shape_button_scale_hover_beats_selection() {
        // Hover wins even when also selected.
        assert_eq!(shape_button_scale(true, true), 1.3);
        assert_eq!(shape_button_scale(true, false), 1.3);
    }

    #[test]
    fn shape_button_scale_selected_only() {
        assert_eq!(shape_button_scale(false, true), 1.15);
    }

    #[test]
    fn shape_button_scale_default() {
        assert_eq!(shape_button_scale(false, false), 1.0);
    }
}
