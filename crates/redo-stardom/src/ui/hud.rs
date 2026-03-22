use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use stardom_core::game::GameCommand;

use crate::game_bridge::GameWorld;
use crate::states::AppState;

use super::display::{office_tier_text, phase_text};

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
                "第 {} 年 / 第 {} 週",
                g.calendar.year, g.calendar.week
            ));
            ui.separator();
            ui.label(format!("${}", g.company.balance.0));
            ui.separator();
            ui.label(format!(
                "辦公室：{}",
                office_tier_text(g.company.office_tier)
            ));
            ui.separator();
            ui.label(format!(
                "藝人：{}/{}",
                g.artists.len(),
                g.company.max_artists
            ));
            ui.separator();
            ui.label(phase_text(g.phase));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("推進一週").clicked() {
                    game.command(GameCommand::AdvanceWeek);
                }
            });
        });
    });
}
