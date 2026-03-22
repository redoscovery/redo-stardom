use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::game_bridge::GameWorld;
use crate::states::AppState;

use super::SelectedArtist;

pub struct DashboardPlugin;

impl Plugin for DashboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            dashboard_ui.run_if(in_state(AppState::InGame)),
        );
    }
}

fn dashboard_ui(
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
            ui.heading("Artists");
            ui.separator();
            if game.0.artists.is_empty() {
                ui.label("No artists signed yet.");
                ui.label("Use Recruitment to scout talent.");
            }
            for (i, artist) in game.0.artists.iter().enumerate() {
                let is_selected = selected.0 == Some(i);
                let text = format!(
                    "{} (Age {})\n  Pop: {} | Stress: {} | {:?}",
                    artist.name,
                    artist.age,
                    artist.stats.popularity,
                    artist.stats.stress,
                    artist.current_activity,
                );
                if ui.selectable_label(is_selected, text).clicked() {
                    selected.0 = Some(i);
                }
            }
        });
}
