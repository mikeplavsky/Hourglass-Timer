pub mod color_panel;
pub mod timer_panel;
pub mod shape_panel;

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

// Resource to track timer panel visibility
#[derive(Resource)]
pub struct TimerPanelVisible(pub bool);

impl Default for TimerPanelVisible {
    fn default() -> Self {
        Self(false) // Start collapsed
    }
}

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            color_panel::ColorPanelPlugin,
            timer_panel::TimerPanelPlugin,
            shape_panel::ShapePanelPlugin,
        ))
        .init_resource::<TimerPanelVisible>()
        .add_systems(Startup, setup_ui_layout);
    }
}

fn setup_ui_layout(mut commands: Commands) {
    // Root UI container - vertical layout
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
        // Top controls container
        parent.spawn((
            Name::new("Top Controls Container"),
            TopControlsMarker,
            Node {
                width: Val::Percent(100.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
        )).with_children(|parent| {
            // Color selection row
            parent.spawn((
                Name::new("Color Row Container"),
                ColorRowMarker,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(60.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    padding: UiRect::vertical(Val::Px(5.0)),
                    overflow: Overflow::clip_x(),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));

            // Shape selection row
            parent.spawn((
                Name::new("Shape Row Container"),
                ShapeRowMarker,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(80.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    padding: UiRect::vertical(Val::Px(5.0)),
                    overflow: Overflow::clip_x(),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ));
        });

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
