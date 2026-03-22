use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::states::AppState;

#[derive(Clone, Debug)]
pub struct ReportEntry {
    pub artist_name: String,
    pub activity: String,
    pub money_change: i64,
    pub stat_changes: Vec<(String, String)>,
}

#[derive(Resource)]
pub struct WeekReport {
    pub entries: Vec<ReportEntry>,
    pub total_income: i64,
    pub total_expenses: i64,
}

pub struct WeekReportPlugin;

impl Plugin for WeekReportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            week_report_ui.run_if(in_state(AppState::InGame).and(resource_exists::<WeekReport>)),
        );
    }
}

fn week_report_ui(mut contexts: EguiContexts, report: Res<WeekReport>, mut commands: Commands) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let mut close = false;

    egui::Window::new("-- 本週結算 --")
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .resizable(false)
        .min_width(400.0)
        .show(ctx, |ui| {
            for entry in &report.entries {
                ui.group(|ui| {
                    ui.heading(&entry.artist_name);
                    ui.label(format!("活動：{}", entry.activity));
                    if entry.money_change != 0 {
                        let prefix = if entry.money_change > 0 {
                            "收入"
                        } else {
                            "支出"
                        };
                        ui.label(format!("{}：${}", prefix, entry.money_change.abs()));
                    }
                    for (stat, change) in &entry.stat_changes {
                        ui.label(format!("  {} {}", stat, change));
                    }
                });
            }

            ui.separator();
            ui.label(format!("本週總收入：${}", report.total_income));
            ui.label(format!("本週總支出：${}", report.total_expenses));
            ui.label(format!(
                "淨變動：${}",
                report.total_income - report.total_expenses
            ));

            ui.add_space(10.0);
            if ui.button("確定").clicked() {
                close = true;
            }
        });

    if close {
        commands.remove_resource::<WeekReport>();
    }
}
