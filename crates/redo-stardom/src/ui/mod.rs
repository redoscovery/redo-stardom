use bevy::prelude::*;

pub mod main_menu;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(main_menu::MainMenuPlugin);
    }
}
