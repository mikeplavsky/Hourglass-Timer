use bevy::prelude::*;
use crate::resources::{HourglassConfig, HourglassShape};
use crate::ui::RightPanelMarker;

pub struct ShapePanelPlugin;

impl Plugin for ShapePanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_shape_buttons)
            .add_systems(Update, handle_shape_button_clicks);
    }
}



#[derive(Component)]
struct ShapeButton {
    shape: HourglassShape,
}

fn spawn_shape_buttons(
    mut commands: Commands,
    query: Query<Entity, With<RightPanelMarker>>,
) {
    // Find the right panel container
    if let Ok(entity) = query.single() {
            commands.entity(entity).with_children(|parent| {
                // Add a label
                parent.spawn((
                    Text::new("Shapes"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                ));

                // Add shape buttons with placeholder mini-hourglasses
                let shapes = [
                    (HourglassShape::Classic, "Classic"),
                    (HourglassShape::Modern, "Modern"),
                    (HourglassShape::Slim, "Slim"),
                    (HourglassShape::Wide, "Wide"),
                ];

                for (shape, label) in shapes {
                    parent.spawn((
                        Name::new(format!("Shape Button {}", label)),
                        ShapeButton { shape },
                        Button,
                        Node {
                            width: Val::Px(60.0),
                            height: Val::Px(80.0),
                            margin: UiRect::all(Val::Px(5.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                        BorderColor(Color::WHITE),
                    )).with_children(|parent| {
                        // Placeholder hourglass representation (temporary - we'll add real mini-hourglasses later)
                        parent.spawn((
                            Node {
                                width: Val::Px(30.0),
                                height: Val::Px(40.0),
                                margin: UiRect::bottom(Val::Px(5.0)),
                                ..default()
                            },
                            BackgroundColor(match shape {
                                HourglassShape::Classic => Color::srgb(0.6, 0.5, 0.3),
                                HourglassShape::Modern => Color::srgb(0.3, 0.5, 0.6),
                                HourglassShape::Slim => Color::srgb(0.5, 0.3, 0.6),
                                HourglassShape::Wide => Color::srgb(0.6, 0.3, 0.3),
                            }),
                        ));

                        // Label
                        parent.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                }
            });
    }
}

fn handle_shape_button_clicks(
    mut interaction_query: Query<
        (&Interaction, &ShapeButton, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut config: ResMut<HourglassConfig>,
) {
    for (interaction, shape_button, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                config.shape_type = shape_button.shape;
                *border_color = BorderColor(Color::srgb(0.0, 1.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
            }
            Interaction::None => {
                // Highlight selected shape
                if config.shape_type == shape_button.shape {
                    *border_color = BorderColor(Color::srgb(0.0, 0.8, 0.0));
                } else {
                    *border_color = BorderColor(Color::WHITE);
                }
            }
        }
    }
}
