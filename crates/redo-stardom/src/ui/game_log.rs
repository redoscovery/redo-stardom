use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::game_bridge::GameWorld;

pub fn game_log_ui(mut contexts: EguiContexts, game: Res<GameWorld>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::TopBottomPanel::bottom("game_log").show(ctx, |ui| {
        ui.label("遊戲日誌");
        ui.separator();
        egui::ScrollArea::vertical()
            .max_height(120.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                let log = &game.0.log;
                // Show most recent entries (last 100)
                let start = log.len().saturating_sub(100);
                for entry in &log[start..] {
                    ui.label(entry);
                }
                if log.is_empty() {
                    ui.weak("（尚無紀錄）");
                }
            });
    });
}
