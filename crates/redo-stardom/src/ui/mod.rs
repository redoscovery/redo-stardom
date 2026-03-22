use bevy::prelude::*;

pub mod artist_panel;
pub mod dashboard;
pub mod display;
pub mod events;
pub mod gig_market;
pub mod hud;
pub mod main_menu;

#[derive(Resource, Default)]
pub struct SelectedArtist(pub Option<usize>);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedArtist>();
        app.add_plugins(main_menu::MainMenuPlugin);
        app.add_plugins(hud::HudPlugin);
        app.add_plugins(dashboard::DashboardPlugin);
        app.add_plugins(artist_panel::ArtistPanelPlugin);
        app.add_plugins(gig_market::GigMarketPlugin);
        app.add_plugins(events::EventsPlugin);
    }
}
