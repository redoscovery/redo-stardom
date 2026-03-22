use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::EguiPlugin;

mod data_loading;
mod game_bridge;
mod states;
mod ui;

use states::AppState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "REDÓ Stardom".to_string(),
                resolution: WindowResolution::new(800, 600),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .init_state::<AppState>()
        .add_plugins(ui::UiPlugin)
        .run();
}
