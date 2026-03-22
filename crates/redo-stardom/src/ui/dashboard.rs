use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::game_bridge::GameWorld;

use super::SelectedArtist;
use super::display::activity_text;

pub fn dashboard_ui(
    mut contexts: EguiContexts,
    game: Res<GameWorld>,
    mut selected: ResMut<SelectedArtist>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    egui::SidePanel::left("roster")
        .min_width(220.0)
        .show(ctx, |ui| {
            ui.heading("藝人名冊");
            ui.separator();
            if game.0.artists.is_empty() {
                ui.label("尚未簽約任何藝人。");
                ui.label("前往招募頁面發掘人才。");
            }
            for (i, artist) in game.0.artists.iter().enumerate() {
                let is_selected = selected.0 == Some(i);
                let text = format!(
                    "{} ({}歲)\n  人氣：{} | 壓力：{} | {}",
                    artist.name,
                    artist.age,
                    artist.stats.popularity,
                    artist.stats.stress,
                    activity_text(&artist.current_activity),
                );
                if ui.selectable_label(is_selected, text).clicked() {
                    selected.0 = Some(i);
                }
            }
        });
}
