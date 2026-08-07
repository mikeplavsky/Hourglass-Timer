pub mod color_panel;
pub mod pause_overlay;
pub mod shape_panel;
pub mod timer_panel;

use crate::resources::{AppearanceStateChanged, PendingFlip};
use crate::timer::TimerCommand;
use bevy::prelude::*;

pub struct UIPlugin;

#[cfg(feature = "chrome_extension")]
pub(crate) const SIDEBAR_APPEARANCE_PADDING: f32 = 4.0;
#[cfg(feature = "chrome_extension")]
pub(crate) const SIDEBAR_COLOR_ROW_HEIGHT: f32 = 28.0;
#[cfg(feature = "chrome_extension")]
pub(crate) const SIDEBAR_SHAPE_ROW_HEIGHT: f32 = 52.0;

pub(crate) fn extension_appearance_change_command(
    pending_flip: &mut PendingFlip,
) -> Option<TimerCommand> {
    if cfg!(feature = "chrome_extension") {
        pending_flip.0 = true;
        Some(TimerCommand::Restart)
    } else {
        None
    }
}

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
struct AppearanceControlsContainer;

// Resource to track timer panel visibility
#[derive(Resource, Default)]
pub struct TimerPanelVisible(pub bool);

#[derive(Resource)]
pub struct AppearancePanelVisible(pub bool);

impl Default for AppearancePanelVisible {
    fn default() -> Self {
        Self(true)
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
                        padding: UiRect::all(Val::Px(SIDEBAR_APPEARANCE_PADDING)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Name::new("Appearance Controls"),
                            AppearanceControlsContainer,
                            Node {
                                width: Val::Percent(100.0),
                                display: Display::Flex,
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
                                    min_height: Val::Px(SIDEBAR_COLOR_ROW_HEIGHT),
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    flex_wrap: FlexWrap::Wrap,
                                    align_content: AlignContent::Center,
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    padding: UiRect::all(Val::Px(3.0)),
                                    row_gap: Val::Px(2.0),
                                    ..default()
                                },
                            ));
                            parent.spawn((
                                Name::new("Sidebar Shape Row"),
                                ShapeRowMarker,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(SIDEBAR_SHAPE_ROW_HEIGHT),
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

    #[test]
    fn sidebar_appearance_controls_are_always_visible_without_toggle() {
        let mut app = App::new();
        app.add_systems(Startup, setup_sidebar_ui_layout);
        app.update();

        assert!(AppearancePanelVisible::default().0);
        let world = app.world_mut();
        let mut controls_query = world.query_filtered::<&Node, With<AppearanceControlsContainer>>();
        assert_eq!(controls_query.single(world).unwrap().display, Display::Flex);

        let mut names = world.query::<&Name>();
        assert!(
            names
                .iter(world)
                .all(|name| name.as_str() != "Appearance Toggle")
        );
    }

    #[test]
    fn sidebar_color_row_can_grow_when_controls_wrap() {
        let mut app = App::new();
        app.add_systems(Startup, setup_sidebar_ui_layout);
        app.update();

        let world = app.world_mut();
        let mut query = world.query_filtered::<&Node, With<ColorRowMarker>>();
        let row = query.single(world).unwrap();
        assert_eq!(row.height, Val::Auto);
        assert_eq!(row.min_height, Val::Px(SIDEBAR_COLOR_ROW_HEIGHT));
        assert_eq!(row.flex_wrap, FlexWrap::Wrap);
        assert_eq!(row.overflow, Overflow::visible());
    }
}

#[cfg(test)]
mod appearance_change_tests {
    use super::*;

    #[test]
    fn appearance_restart_and_flip_are_extension_only() {
        let mut pending_flip = PendingFlip(false);
        let command = extension_appearance_change_command(&mut pending_flip);

        if cfg!(feature = "chrome_extension") {
            assert_eq!(command, Some(TimerCommand::Restart));
            assert!(pending_flip.0);
        } else {
            assert_eq!(command, None);
            assert!(!pending_flip.0);
        }
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
