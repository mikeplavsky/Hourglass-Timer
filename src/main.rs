// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]

mod hourglass;
pub mod resources;
mod timer;
mod ui;

use bevy::prelude::*;
use resources::{HourglassConfig, TimerState};

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Window {
                    title: "Hourglass Timer".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }
                .into(),
                ..default()
            }),
        );

        // Initialize resources
        app.init_resource::<HourglassConfig>()
            .init_resource::<TimerState>();

        // Add our custom plugins
        app.add_plugins((hourglass::HourglassPlugin, timer::TimerPlugin, ui::UIPlugin));

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Name::new("Camera"), Camera2d));
}
