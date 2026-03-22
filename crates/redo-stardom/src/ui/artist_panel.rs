use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use stardom_core::game::GameCommand;
use stardom_core::types::Activity;

use crate::data_loading::GameCatalogs;
use crate::game_bridge::GameWorld;
use crate::states::AppState;

use super::SelectedArtist;

pub struct ArtistPanelPlugin;

impl Plugin for ArtistPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, artist_panel_ui.run_if(in_state(AppState::InGame)));
    }
}

fn artist_panel_ui(
    mut contexts: EguiContexts,
    mut game: ResMut<GameWorld>,
    selected: Res<SelectedArtist>,
    catalogs: Option<Res<GameCatalogs>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Snapshot artist data before entering egui closures to avoid borrow conflicts
    let Some(idx) = selected.0 else {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.label("Select an artist from the roster.");
            });
        });
        return;
    };

    let Some(artist) = game.0.artists.get(idx).cloned() else {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Invalid artist selection.");
        });
        return;
    };

    // Snapshot catalogs
    let training_defs = catalogs
        .as_ref()
        .map(|c| c.training.clone())
        .unwrap_or_default();
    let job_defs = catalogs
        .as_ref()
        .map(|c| c.jobs.clone())
        .unwrap_or_default();

    // Collect all pending commands, then apply after UI rendering
    let mut pending_cmd: Option<GameCommand> = None;

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(&artist.name);
        ui.label(format!(
            "Age: {} | Activity: {:?}",
            artist.age, artist.current_activity
        ));
        if artist.is_locked() {
            ui.label(format!(
                "Locked in gig for {} more weeks",
                artist.locked_weeks
            ));
        }
        ui.separator();

        // Two-column layout
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.columns(2, |cols| {
                // Left column: attributes and stats
                cols[0].heading("Base Attributes");
                cols[0].label(format!("Stamina: {}", artist.base_attributes.stamina));
                cols[0].label(format!("Intellect: {}", artist.base_attributes.intellect));
                cols[0].label(format!("Empathy: {}", artist.base_attributes.empathy));
                cols[0].label(format!("Charm: {}", artist.base_attributes.charm));
                cols[0].add_space(8.0);

                cols[0].heading("Professional Skills");
                let skills = &artist.skills;
                for (name, val) in [
                    ("Vocal", skills.vocal),
                    ("Acting", skills.acting),
                    ("Dance", skills.dance),
                    ("Poise", skills.poise),
                    ("Eloquence", skills.eloquence),
                    ("Creativity", skills.creativity),
                ] {
                    cols[0].horizontal(|ui| {
                        ui.label(format!("{name:>10}: {val:>5}"));
                        ui.add(egui::ProgressBar::new(val as f32 / 10000.0).desired_width(120.0));
                    });
                }
                cols[0].add_space(8.0);

                cols[0].heading("Inner Traits");
                cols[0].label(format!("Confidence: {}", artist.traits.confidence));
                cols[0].label(format!("Rebellion: {}", artist.traits.rebellion));
                cols[0].add_space(8.0);

                cols[0].heading("Personality");
                cols[0].label(format!(
                    "Social: {} ({})",
                    artist.personality.social,
                    if artist.personality.social < 0 {
                        "Introvert"
                    } else {
                        "Extrovert"
                    }
                ));
                cols[0].label(format!(
                    "Thinking: {} ({})",
                    artist.personality.thinking,
                    if artist.personality.thinking < 0 {
                        "Intuitive"
                    } else {
                        "Logical"
                    }
                ));
                cols[0].label(format!(
                    "Action: {} ({})",
                    artist.personality.action,
                    if artist.personality.action < 0 {
                        "Cautious"
                    } else {
                        "Adventurous"
                    }
                ));
                cols[0].label(format!(
                    "Stance: {} ({})",
                    artist.personality.stance,
                    if artist.personality.stance < 0 {
                        "Easygoing"
                    } else {
                        "Competitive"
                    }
                ));

                // Right column: image tags, aux stats, activity assignment
                cols[1].heading("Image Tags");
                let img = &artist.image;
                for (name, val) in [
                    ("Pure", img.pure),
                    ("Sexy", img.sexy),
                    ("Cool", img.cool),
                    ("Intellectual", img.intellectual),
                    ("Funny", img.funny),
                    ("Mysterious", img.mysterious),
                ] {
                    cols[1].label(format!("{name}: {val}"));
                }
                cols[1].add_space(8.0);

                cols[1].heading("Market Status");
                cols[1].label(format!(
                    "Recognition: {} ({:?})",
                    artist.stats.recognition,
                    artist.stats.recognition_tier()
                ));
                cols[1].label(format!("Reputation: {}", artist.stats.reputation));
                cols[1].label(format!("Popularity: {}", artist.stats.popularity));
                cols[1].label(format!("Stress: {}", artist.stats.stress));
                cols[1].add_space(16.0);

                // Activity assignment
                cols[1].heading("Assign Activity");
                let is_busy = artist.is_locked() || artist.current_activity != Activity::Idle;
                if is_busy {
                    cols[1].label("Artist is busy this week.");
                } else {
                    // Training options from catalogs
                    for training in &training_defs {
                        let cost = training.tiers.first().map(|t| t.cost).unwrap_or(0);
                        let label = format!("{} (${cost})", training.name);
                        if cols[1].button(&label).clicked() {
                            pending_cmd = Some(GameCommand::AssignTraining {
                                artist_index: idx,
                                training: training.clone(),
                            });
                        }
                    }

                    cols[1].add_space(4.0);

                    // Job options from catalogs
                    for job in &job_defs {
                        let label = format!("{} (+${})", job.name, job.pay);
                        if cols[1].button(&label).clicked() {
                            pending_cmd = Some(GameCommand::AssignJob {
                                artist_index: idx,
                                job: job.clone(),
                            });
                        }
                    }

                    cols[1].add_space(4.0);
                    if cols[1].button("Rest").clicked() {
                        pending_cmd = Some(GameCommand::AssignRest { artist_index: idx });
                    }
                }
            });
        });
    });

    // Apply any pending command after UI rendering is done
    if let Some(cmd) = pending_cmd {
        game.command(cmd);
    }
}
