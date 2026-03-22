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

        // Two-column layout
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.columns(2, |cols| {
                // Left column: attributes and stats
                cols[0].heading("基礎屬性");
                cols[0].label(format!("體能：{}", artist.base_attributes.stamina));
                cols[0].label(format!("智識：{}", artist.base_attributes.intellect));
                cols[0].label(format!("共情：{}", artist.base_attributes.empathy));
                cols[0].label(format!("魅力：{}", artist.base_attributes.charm));
                cols[0].add_space(8.0);

                cols[0].heading("專業技能");
                let skills = &artist.skills;
                for (name, val) in [
                    ("歌藝", skills.vocal),
                    ("演技", skills.acting),
                    ("舞藝", skills.dance),
                    ("儀態", skills.poise),
                    ("口才", skills.eloquence),
                    ("創作", skills.creativity),
                ] {
                    cols[0].horizontal(|ui| {
                        ui.label(format!("{name:>4}：{val:>5}"));
                        ui.add(egui::ProgressBar::new(val as f32 / 10000.0).desired_width(120.0));
                    });
                }
                cols[0].add_space(8.0);

                cols[0].heading("內在特質");
                cols[0].label(format!("自信：{}", artist.traits.confidence));
                cols[0].label(format!("叛逆：{}", artist.traits.rebellion));
                cols[0].add_space(8.0);

                cols[0].heading("性格光譜");
                cols[0].label(format!(
                    "社交：{} ({})",
                    artist.personality.social,
                    if artist.personality.social < 0 {
                        "內斂"
                    } else {
                        "外放"
                    }
                ));
                cols[0].label(format!(
                    "思維：{} ({})",
                    artist.personality.thinking,
                    if artist.personality.thinking < 0 {
                        "直覺"
                    } else {
                        "邏輯"
                    }
                ));
                cols[0].label(format!(
                    "行事：{} ({})",
                    artist.personality.action,
                    if artist.personality.action < 0 {
                        "謹慎"
                    } else {
                        "冒險"
                    }
                ));
                cols[0].label(format!(
                    "處世：{} ({})",
                    artist.personality.stance,
                    if artist.personality.stance < 0 {
                        "隨和"
                    } else {
                        "好勝"
                    }
                ));

                // Right column: image tags, aux stats, activity assignment
                cols[1].heading("形象標籤");
                let img = &artist.image;
                for (name, val) in [
                    ("清純", img.pure),
                    ("性感", img.sexy),
                    ("酷帥", img.cool),
                    ("知性", img.intellectual),
                    ("搞笑", img.funny),
                    ("神秘", img.mysterious),
                ] {
                    cols[1].label(format!("{name}：{val}"));
                }
                cols[1].add_space(8.0);

                cols[1].heading("市場狀態");
                cols[1].label(format!(
                    "知名度：{} ({})",
                    artist.stats.recognition,
                    recognition_tier_text(artist.stats.recognition_tier())
                ));
                cols[1].label(format!("風評：{}", artist.stats.reputation));
                cols[1].label(format!("人氣：{}", artist.stats.popularity));
                cols[1].label(format!("壓力：{}", artist.stats.stress));
                cols[1].add_space(16.0);

                // Activity assignment
                cols[1].heading("安排活動");
                if artist.is_locked() {
                    cols[1].label(format!(
                        "[鎖定] 通告中（剩餘 {} 週）",
                        artist.locked_weeks
                    ));
                } else if let Some(label) = &current_plan_label {
                    // Already have a planned activity — show it with cancel option
                    cols[1].label(format!("[已安排] {}", label));
                    if cols[1].button("取消").clicked() {
                        cancel_plan = true;
                    }
                } else {
                    // No plan yet — show activity buttons
                    // Training options from catalogs
                    for training in &training_defs {
                        let cost = training.tiers.first().map(|t| t.cost).unwrap_or(0);
                        let label = format!("{} (${cost})", training.name);
                        if cols[1].button(&label).clicked() {
                            new_plan = Some(PlannedActivity::Training(training.clone()));
                        }
                    }

                    cols[1].add_space(4.0);

                    // Job options from catalogs
                    for job in &job_defs {
                        let label = format!("{} (+${})", job.name, job.pay);
                        if cols[1].button(&label).clicked() {
                            new_plan = Some(PlannedActivity::Job(job.clone()));
                        }
                    }

                    cols[1].add_space(4.0);
                    if cols[1].button("休息").clicked() {
                        new_plan = Some(PlannedActivity::Rest);
                    }
                }
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
