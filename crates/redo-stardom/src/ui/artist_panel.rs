use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::data_loading::GameCatalogs;
use crate::game_bridge::GameWorld;
use crate::states::AppState;

use super::SelectedArtist;
use super::display::{activity_text, recognition_tier_text};
use super::week_plan::{PlannedActivity, WeekPlan};
use super::week_report::WeekReport;

pub struct ArtistPanelPlugin;

impl Plugin for ArtistPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            artist_panel_ui.run_if(in_state(AppState::InGame).and(not(resource_exists::<WeekReport>))),
        );
    }
}

fn artist_panel_ui(
    mut contexts: EguiContexts,
    game: Res<GameWorld>,
    selected: Res<SelectedArtist>,
    catalogs: Option<Res<GameCatalogs>>,
    mut week_plan: ResMut<WeekPlan>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Snapshot artist data before entering egui closures to avoid borrow conflicts
    let Some(idx) = selected.0 else {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.label("請從藝人名冊選擇一位藝人。");
            });
        });
        return;
    };

    let Some(artist) = game.0.artists.get(idx).cloned() else {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("選擇無效。");
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

    // Snapshot current plan for this artist
    let current_plan_label = week_plan.get(idx).map(|a| a.label());

    // Collect planned activity to assign, or cancel flag
    let mut new_plan: Option<PlannedActivity> = None;
    let mut cancel_plan = false;

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(&artist.name);
        ui.label(format!(
            "年齡：{} | 活動：{}",
            artist.age,
            activity_text(&artist.current_activity)
        ));
        if artist.is_locked() {
            ui.label(format!("[鎖定] 通告進行中，剩餘 {} 週", artist.locked_weeks));
        }
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Activity assignment — always visible at top
            ui.heading("安排活動");
            if artist.is_locked() {
                ui.label(format!("[鎖定] 通告中（剩餘 {} 週）", artist.locked_weeks));
            } else if let Some(label) = &current_plan_label {
                ui.horizontal(|ui| {
                    ui.label(format!("[已安排] {}", label));
                    if ui.button("取消").clicked() {
                        cancel_plan = true;
                    }
                });
            } else {
                ui.horizontal_wrapped(|ui| {
                    for training in &training_defs {
                        let cost = training.tiers.first().map(|t| t.cost).unwrap_or(0);
                        if ui.button(format!("{} (${})", training.name, cost)).clicked() {
                            new_plan = Some(PlannedActivity::Training(training.clone()));
                        }
                    }
                    for job in &job_defs {
                        if ui.button(format!("{} (+${})", job.name, job.pay)).clicked() {
                            new_plan = Some(PlannedActivity::Job(job.clone()));
                        }
                    }
                    if ui.button("休息").clicked() {
                        new_plan = Some(PlannedActivity::Rest);
                    }
                });
            }
            ui.separator();

            // Stats in collapsible sections
            egui::CollapsingHeader::new("基礎屬性").default_open(true).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("體能：{}",  artist.base_attributes.stamina));
                    ui.label(format!("智識：{}",  artist.base_attributes.intellect));
                    ui.label(format!("共情：{}",  artist.base_attributes.empathy));
                    ui.label(format!("魅力：{}",  artist.base_attributes.charm));
                });
            });

            egui::CollapsingHeader::new("專業技能").default_open(true).show(ui, |ui| {
                let skills = &artist.skills;
                for (name, val) in [
                    ("歌藝", skills.vocal), ("演技", skills.acting),
                    ("舞藝", skills.dance), ("儀態", skills.poise),
                    ("口才", skills.eloquence), ("創作", skills.creativity),
                ] {
                    ui.horizontal(|ui| {
                        ui.label(format!("{name}：{val:>5}"));
                        ui.add(egui::ProgressBar::new(val as f32 / 10000.0).desired_width(100.0));
                    });
                }
            });

            egui::CollapsingHeader::new("內在特質").default_open(false).show(ui, |ui| {
                ui.label(format!("自信：{}", artist.traits.confidence));
                ui.label(format!("叛逆：{}", artist.traits.rebellion));
            });

            egui::CollapsingHeader::new("性格光譜").default_open(false).show(ui, |ui| {
                ui.label(format!("社交：{} ({})", artist.personality.social,
                    if artist.personality.social < 0 { "內斂" } else { "外放" }));
                ui.label(format!("思維：{} ({})", artist.personality.thinking,
                    if artist.personality.thinking < 0 { "直覺" } else { "邏輯" }));
                ui.label(format!("行事：{} ({})", artist.personality.action,
                    if artist.personality.action < 0 { "謹慎" } else { "冒險" }));
                ui.label(format!("處世：{} ({})", artist.personality.stance,
                    if artist.personality.stance < 0 { "隨和" } else { "好勝" }));
            });

            egui::CollapsingHeader::new("形象標籤").default_open(false).show(ui, |ui| {
                let img = &artist.image;
                ui.horizontal_wrapped(|ui| {
                    for (name, val) in [
                        ("清純", img.pure), ("性感", img.sexy), ("酷帥", img.cool),
                        ("知性", img.intellectual), ("搞笑", img.funny), ("神秘", img.mysterious),
                    ] {
                        ui.label(format!("{name}：{val}"));
                    }
                });
            });

            egui::CollapsingHeader::new("市場狀態").default_open(true).show(ui, |ui| {
                ui.label(format!("知名度：{} ({})", artist.stats.recognition,
                    recognition_tier_text(artist.stats.recognition_tier())));
                ui.label(format!("風評：{}", artist.stats.reputation));
                ui.label(format!("人氣：{}", artist.stats.popularity));
                ui.label(format!("壓力：{}", artist.stats.stress));
            });
        });
    });

    // Apply plan changes after UI rendering
    if cancel_plan {
        week_plan.cancel(idx);
    } else if let Some(activity) = new_plan {
        week_plan.assign(idx, activity);
    }
}
