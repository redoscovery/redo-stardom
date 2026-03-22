use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use stardom_core::game::GameCommand;

use crate::game_bridge::GameWorld;
use crate::states::AppState;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            hud_ui.run_if(in_state(AppState::InGame)),
        );
    }
}

fn hud_ui(mut contexts: EguiContexts, mut game: ResMut<GameWorld>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    egui::TopBottomPanel::top("hud").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let g = &game.0;
            ui.label(format!(
                "Year {} / Week {}",
                g.calendar.year, g.calendar.week
            ));
            ui.separator();
            ui.label(format!("${}", g.company.balance.0));
            ui.separator();
            ui.label(format!("Office: {:?}", g.company.office_tier));
            ui.separator();
            ui.label(format!(
                "Artists: {}/{}",
                g.artists.len(),
                g.company.max_artists
            ));
            ui.separator();
            ui.label(format!("{:?}", g.phase));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Advance Week").clicked() {
                    game.command(GameCommand::AdvanceWeek);
                }
            });
        });
    });
}
