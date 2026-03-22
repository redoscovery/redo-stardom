use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use crate::states::AppState;

pub mod central_tabs;
pub mod dashboard;
pub mod display;
pub mod events;
pub mod game_log;
pub mod hud;
pub mod main_menu;
pub mod week_plan;
pub mod week_report;

#[derive(Resource, Default)]
pub struct SelectedArtist(pub Option<usize>);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedArtist>();
        app.init_resource::<week_plan::WeekPlan>();
        app.init_resource::<central_tabs::ActiveTab>();

        // Main menu (only in MainMenu state)
        app.add_plugins(main_menu::MainMenuPlugin);

        // InGame UI — MUST be chained for correct egui panel order:
        // TopPanel → BottomPanel → SidePanel → CentralPanel
        app.add_systems(
            EguiPrimaryContextPass,
            (
                hud::hud_ui,
                game_log::game_log_ui,
                dashboard::dashboard_ui,
                // CentralPanel OR modal (events/report) — these exclude each other via internal guards
                central_tabs::central_tabs_ui,
                events::events_ui,
                week_report::week_report_ui,
            )
                .chain()
                .run_if(in_state(AppState::InGame)),
        );
    }
}
