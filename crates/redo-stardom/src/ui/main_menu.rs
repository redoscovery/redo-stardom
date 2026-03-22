use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::game_bridge::GameWorld;
use crate::states::AppState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            main_menu_ui.run_if(in_state(AppState::MainMenu)),
        );
    }
}

fn main_menu_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(150.0);
            ui.heading(egui::RichText::new("REDÓ Stardom").size(48.0));
            ui.add_space(10.0);
            ui.label(egui::RichText::new("明星經紀模擬遊戲").size(16.0).weak());
            ui.add_space(40.0);

            if ui
                .button(egui::RichText::new("  新遊戲  ").size(24.0))
                .clicked()
            {
                let (game_world, catalogs) = GameWorld::new_game();
                commands.insert_resource(game_world);
                commands.insert_resource(catalogs);
                next_state.set(AppState::InGame);
            }
            ui.add_space(10.0);
            if ui
                .button(egui::RichText::new("    結束    ").size(24.0))
                .clicked()
            {
                std::process::exit(0);
            }
        });
    });
}
