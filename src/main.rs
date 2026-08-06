// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]

#[cfg(feature = "chrome_extension")]
mod chrome_extension;
mod hourglass;
pub mod resources;
mod timer;
mod ui;

use bevy::prelude::*;
use resources::{HourglassConfig, TimerState};

fn main() -> AppExit {
    #[cfg(all(feature = "chrome_extension", target_arch = "wasm32"))]
    chrome_extension::report_startup_stage("Rust module started…");

    let mut app = App::new();

    #[cfg(all(feature = "chrome_extension", target_arch = "wasm32"))]
    chrome_extension::report_startup_stage("Configuring Bevy…");

    app.add_plugins(AppPlugin);

    #[cfg(all(feature = "chrome_extension", target_arch = "wasm32"))]
    chrome_extension::report_startup_stage("Launching Bevy…");

    app.run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Window {
                    title: "Hourglass Timer".to_string(),
                    #[cfg(feature = "chrome_extension")]
                    canvas: Some("#hourglass-canvas".to_string()),
                    fit_canvas_to_parent: true,
                    ..default()
                }
                .into(),
                ..default()
            }),
        );

        #[cfg(all(feature = "chrome_extension", target_arch = "wasm32"))]
        chrome_extension::report_startup_stage("Bevy platform configured…");

        // Initialize resources
        app.init_resource::<HourglassConfig>()
            .init_resource::<TimerState>();

        // Add our custom plugins
        app.add_plugins((hourglass::HourglassPlugin, timer::TimerPlugin, ui::UIPlugin));

        #[cfg(all(feature = "chrome_extension", target_arch = "wasm32"))]
        chrome_extension::report_startup_stage("Hourglass systems configured…");

        #[cfg(all(feature = "chrome_extension", target_arch = "wasm32"))]
        app.add_plugins(chrome_extension::ChromeExtensionPlugin);

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Name::new("Camera"), Camera2d));
}
