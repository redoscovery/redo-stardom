use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use stardom_core::game::GameCommand;

use crate::game_bridge::GameWorld;
use crate::states::AppState;

use super::display::{office_tier_text, phase_text};
use super::week_plan::{PlannedActivity, WeekPlan};
use super::week_report::{ReportEntry, WeekReport};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            hud_ui.run_if(in_state(AppState::InGame).and(not(resource_exists::<WeekReport>))),
        );
    }
}

fn hud_ui(
    mut contexts: EguiContexts,
    mut game: ResMut<GameWorld>,
    mut week_plan: ResMut<WeekPlan>,
    mut commands: Commands,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Determine which artist indices are non-locked (need assignments)
    let non_locked_indices: Vec<usize> = game
        .0
        .artists
        .iter()
        .enumerate()
        .filter(|(_, a)| !a.is_locked())
        .map(|(i, _)| i)
        .collect();

    let all_assigned = week_plan.all_assigned(&non_locked_indices);
    let artist_count = game.0.artists.len();

    let assigned_count = non_locked_indices
        .iter()
        .filter(|i| week_plan.get(**i).is_some())
        .count();
    let need_count = non_locked_indices.len();

    let mut advance = false;

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
            ui.label(format!("藝人：{}/{}", artist_count, g.company.max_artists));
            ui.separator();
            ui.label(phase_text(g.phase));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let btn =
                    egui::Button::new(format!("推進一週 ({}/{})", assigned_count, need_count));
                if all_assigned {
                    if ui.add(btn).clicked() {
                        advance = true;
                    }
                } else {
                    ui.add_enabled(false, btn);
                }
            });
        });
    });

    if advance {
        let report = execute_week(&mut game, &week_plan);
        commands.insert_resource(report);
        week_plan.clear();
    }
}

fn execute_week(game: &mut ResMut<GameWorld>, plan: &WeekPlan) -> WeekReport {
    // Snapshot before executing
    let snapshots: Vec<_> = game
        .0
        .artists
        .iter()
        .map(|a| (a.name.clone(), a.skills, a.stats))
        .collect();
    let balance_before = game.0.company.balance.0;

    // Execute planned activities
    for (idx, activity) in &plan.assignments {
        match activity {
            PlannedActivity::Training(t) => game.command(GameCommand::AssignTraining {
                artist_index: *idx,
                training: t.clone(),
            }),
            PlannedActivity::Job(j) => game.command(GameCommand::AssignJob {
                artist_index: *idx,
                job: j.clone(),
            }),
            PlannedActivity::Gig(g) => game.command(GameCommand::AssignGig {
                artist_index: *idx,
                gig: g.clone(),
            }),
            PlannedActivity::Rest => game.command(GameCommand::AssignRest { artist_index: *idx }),
        }
    }

    // Advance the week
    game.command(GameCommand::AdvanceWeek);

    // Build report by comparing before/after
    let balance_after = game.0.company.balance.0;
    let net = balance_after - balance_before;

    let mut entries = Vec::new();

    for (idx, (name, old_skills, old_stats)) in snapshots.iter().enumerate() {
        if let Some(artist) = game.0.artists.get(idx) {
            let activity_label = plan
                .assignments
                .get(&idx)
                .map(|a| a.label())
                .unwrap_or_else(|| "(通告中)".to_string());

            let mut stat_changes = Vec::new();

            // Skill changes
            let skill_pairs = [
                ("歌藝", old_skills.vocal, artist.skills.vocal),
                ("演技", old_skills.acting, artist.skills.acting),
                ("舞藝", old_skills.dance, artist.skills.dance),
                ("儀態", old_skills.poise, artist.skills.poise),
                ("口才", old_skills.eloquence, artist.skills.eloquence),
                ("創作", old_skills.creativity, artist.skills.creativity),
            ];
            for (skill_name, old, new) in skill_pairs {
                let diff = new - old;
                if diff != 0 {
                    stat_changes.push((skill_name.to_string(), format!("{:+}", diff)));
                }
            }

            // Stat changes
            let stress_diff = artist.stats.stress - old_stats.stress;
            if stress_diff != 0 {
                stat_changes.push(("壓力".to_string(), format!("{:+}", stress_diff)));
            }
            let pop_diff = artist.stats.popularity - old_stats.popularity;
            if pop_diff != 0 {
                stat_changes.push(("人氣".to_string(), format!("{:+}", pop_diff)));
            }
            let rep_diff = artist.stats.reputation - old_stats.reputation;
            if rep_diff != 0 {
                stat_changes.push(("風評".to_string(), format!("{:+}", rep_diff)));
            }
            let rec_diff = artist.stats.recognition - old_stats.recognition;
            if rec_diff != 0 {
                stat_changes.push(("知名度".to_string(), format!("{:+}", rec_diff)));
            }

            entries.push(ReportEntry {
                artist_name: name.clone(),
                activity: activity_label,
                money_change: 0, // individual breakdown not tracked; see totals
                stat_changes,
            });
        }
    }

    let total_income = net.max(0);
    let total_expenses = (-net).max(0);

    WeekReport {
        entries,
        total_income,
        total_expenses,
    }
}
