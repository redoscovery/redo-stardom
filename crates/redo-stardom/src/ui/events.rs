use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use stardom_core::game::GameCommand;

use crate::game_bridge::GameWorld;

pub fn events_ui(mut contexts: EguiContexts, mut game: ResMut<GameWorld>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Only show if there are active crises
    if game.0.active_crises.is_empty() {
        return;
    }

    // Snapshot crisis data before entering egui closures
    let crises: Vec<_> = game
        .0
        .active_crises
        .iter()
        .enumerate()
        .map(|(i, (artist_idx, crisis))| {
            let artist_name = game
                .0
                .artists
                .get(*artist_idx)
                .map(|a| a.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            (
                i,
                artist_name,
                crisis.name.clone(),
                crisis.description.clone(),
                crisis.choices.clone(),
            )
        })
        .collect();

    let mut pending_response: Option<(usize, usize)> = None;

    egui::Window::new("[緊急] 公關危機")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            for (crisis_idx, artist_name, name, description, choices) in &crises {
                ui.heading(name);
                ui.label(format!("受影響藝人：{}", artist_name));
                ui.label(description.as_str());
                ui.separator();
                for (choice_idx, choice) in choices.iter().enumerate() {
                    let label = format!(
                        "{} (風評 {:+}，人氣 {:+}，壓力 {:+})",
                        choice.label,
                        choice.reputation_change,
                        choice.popularity_change,
                        choice.stress_change,
                    );
                    if ui.button(label).clicked() {
                        pending_response = Some((*crisis_idx, choice_idx));
                    }
                }
                ui.separator();
            }
        });

    if let Some((crisis_index, choice)) = pending_response {
        game.command(GameCommand::RespondToCrisis {
            crisis_index,
            choice,
        });
    }
}
