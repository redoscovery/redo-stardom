use bevy::prelude::*;

pub mod central_tabs;
pub mod dashboard;
pub mod display;
pub mod events;
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
        app.add_plugins(main_menu::MainMenuPlugin);
        app.add_plugins(hud::HudPlugin);
        app.add_plugins(dashboard::DashboardPlugin);
        app.add_plugins(central_tabs::CentralTabsPlugin);
        app.add_plugins(events::EventsPlugin);
        app.add_plugins(week_report::WeekReportPlugin);
    }
}
