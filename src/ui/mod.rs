pub mod color_panel;
pub mod pause_overlay;
pub mod shape_panel;
pub mod timer_panel;

use crate::resources::AppearanceStateChanged;
use bevy::prelude::*;

pub struct UIPlugin;

// Marker components for UI panels
#[derive(Component)]
pub struct TopControlsMarker;

#[derive(Component)]
pub struct ColorRowMarker;

#[derive(Component)]
pub struct ShapeRowMarker;

#[derive(Component)]
pub struct BottomTimerMarker;

#[derive(Component)]
struct AppearanceToggleButton;

#[derive(Component)]
struct AppearanceControlsContainer;

// Resource to track timer panel visibility
#[derive(Resource, Default)]
pub struct TimerPanelVisible(pub bool);

#[derive(Resource)]
pub struct AppearancePanelVisible(pub bool);

impl Default for AppearancePanelVisible {
    fn default() -> Self {
        Self(!cfg!(feature = "chrome_extension"))
    }
}

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            color_panel::ColorPanelPlugin,
            timer_panel::TimerPanelPlugin,
            shape_panel::ShapePanelPlugin,
            pause_overlay::PauseOverlayPlugin,
        ))
        .add_event::<AppearanceStateChanged>()
        .init_resource::<TimerPanelVisible>()
        .init_resource::<AppearancePanelVisible>();

        #[cfg(feature = "chrome_extension")]
        app.add_systems(Startup, setup_sidebar_ui_layout);

        #[cfg(not(feature = "chrome_extension"))]
        app.add_systems(Startup, setup_ui_layout);

        app.add_systems(
            Update,
            (handle_appearance_toggle, update_appearance_visibility),
        );
    }
}

#[cfg(feature = "chrome_extension")]
fn setup_sidebar_ui_layout(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Sidebar UI Root"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Name::new("Sidebar Appearance Container"),
                    TopControlsMarker,
                    Node {
                        width: Val::Percent(100.0),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Name::new("Appearance Toggle"),
                            AppearanceToggleButton,
                            Button,
                            Node {
                                width: Val::Px(150.0),
                                height: Val::Px(30.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
                            BorderColor(Color::srgb(0.65, 0.65, 0.65)),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Appearance"),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    parent
                        .spawn((
                            Name::new("Appearance Controls"),
                            AppearanceControlsContainer,
                            Node {
                                width: Val::Percent(100.0),
                                display: Display::None,
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Name::new("Sidebar Color Row"),
                                ColorRowMarker,
                                Node {
                                    width: Val::Percent(100.0),
                                    min_height: Val::Px(48.0),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    flex_wrap: FlexWrap::Wrap,
                                    align_content: AlignContent::Center,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::all(Val::Px(3.0)),
                                    overflow: Overflow::clip(),
                                    ..default()
                                },
                            ));
                            parent.spawn((
                                Name::new("Sidebar Shape Row"),
                                ShapeRowMarker,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(52.0),
                                    ..default()
                                },
                            ));
                        });
                });

            parent.spawn((
                Name::new("Sidebar Hourglass Area"),
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    min_height: Val::Px(150.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));

            parent.spawn((
                Name::new("Sidebar Timer Container"),
                BottomTimerMarker,
                Node {
                    width: Val::Percent(100.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));
        });
}

#[cfg(all(test, feature = "chrome_extension"))]
mod tests {
    use super::*;

    #[test]
    fn sidebar_timer_container_uses_canvas_background() {
        let mut app = App::new();
        app.add_systems(Startup, setup_sidebar_ui_layout);
        app.update();

        let world = app.world_mut();
        let mut query = world.query_filtered::<&BackgroundColor, With<BottomTimerMarker>>();
        assert_eq!(query.single(world).unwrap().0, Color::NONE);
    }
}

fn handle_appearance_toggle(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<AppearanceToggleButton>),
    >,
    mut visible: ResMut<AppearancePanelVisible>,
) {
    for (interaction, mut background) in &mut query {
        match *interaction {
            Interaction::Pressed => {
                visible.0 = !visible.0;
                *background = BackgroundColor(Color::srgb(0.45, 0.45, 0.45));
            }
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgb(0.35, 0.35, 0.35));
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
            }
        }
    }
}

fn update_appearance_visibility(
    visible: Res<AppearancePanelVisible>,
    mut query: Query<&mut Node, With<AppearanceControlsContainer>>,
) {
    if !visible.is_changed() {
        return;
    }
    for mut node in &mut query {
        node.display = if visible.0 {
            Display::Flex
        } else {
            Display::None
        };
    }
}

#[cfg(not(feature = "chrome_extension"))]
fn setup_ui_layout(mut commands: Commands) {
    // Root UI container - vertical layout
    commands
        .spawn((
            Name::new("UI Root"),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            // Top controls container - narrow color panel only
            parent
                .spawn((
                    Name::new("Top Controls Container"),
                    TopControlsMarker,
                    Node {
                        width: Val::Percent(100.0),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    // Color selection row - narrow and centered
                    parent.spawn((
                        Name::new("Color Row Container"),
                        ColorRowMarker,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(25.0),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::vertical(Val::Px(2.0)),
                            overflow: Overflow::clip_x(),
                            ..default()
                        },
                    ));
                });

            // Shape selection row - positioned directly under color panel
            parent.spawn((
                Name::new("Shape Row Container"),
                ShapeRowMarker,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::vertical(Val::Px(2.0)),
                    overflow: Overflow::clip_x(),
                    ..default()
                },
            ));

            // Center area (for hourglass) - takes remaining space
            parent.spawn((
                Name::new("Center Area"),
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    display: Display::Flex,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));

            // Bottom timer container (collapsible)
            parent.spawn((
                Name::new("Bottom Timer Container"),
                BottomTimerMarker,
                Node {
                    width: Val::Percent(100.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));
        });
}
