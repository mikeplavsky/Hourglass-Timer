pub mod color_panel;
pub mod timer_panel;
pub mod shape_panel;

use bevy::prelude::*;

pub struct UIPlugin;

// Marker components for UI panels
#[derive(Component)]
pub struct LeftPanelMarker;

#[derive(Component)]
pub struct TopPanelMarker;

#[derive(Component)]
pub struct RightPanelMarker;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            color_panel::ColorPanelPlugin,
            timer_panel::TimerPanelPlugin,
            shape_panel::ShapePanelPlugin,
        ))
        .add_systems(Startup, setup_ui_layout);
    }
}

fn setup_ui_layout(mut commands: Commands) {
    // Root UI container
    commands.spawn((
        Name::new("UI Root"),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::NONE),
    )).with_children(|parent| {
        // Top panel container
        parent.spawn((
            Name::new("Top Panel Container"),
            TopPanelMarker,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(80.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
        ));

        // Main content area with side panels
        parent.spawn((
            Name::new("Main Content"),
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(Color::NONE),
        )).with_children(|parent| {
            // Left panel container
            parent.spawn((
                Name::new("Left Panel Container"),
                LeftPanelMarker,
                Node {
                    width: Val::Px(100.0),
                    height: Val::Percent(100.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.9)),
            ));

            // Center area (for hourglass)
            parent.spawn((
                Name::new("Center Area"),
                Node {
                    flex_grow: 1.0,
                    height: Val::Percent(100.0),
                    display: Display::Flex,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));

            // Right panel container
            parent.spawn((
                Name::new("Right Panel Container"),
                RightPanelMarker,
                Node {
                    width: Val::Px(100.0),
                    height: Val::Percent(100.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.9)),
            ));
        });
    });
}
