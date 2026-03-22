use bevy::prelude::*;
use stardom_core::config::Settings;
use stardom_core::game::{GameCommand, GameState};

use crate::data_loading::{self, GameCatalogs};

#[derive(Resource)]
pub struct GameWorld(pub GameState);

impl GameWorld {
    pub fn new_game() -> (Self, GameCatalogs) {
        let mut state = GameState::new(Settings::default());

        // Load all catalog data from RON files
        data_loading::load_game_data(&mut state);
        let catalogs = data_loading::load_catalogs();

        // Add a sample artist so the game starts with something to interact with
        let mut sample = stardom_core::artist::Artist::new(
            stardom_core::types::ArtistId(1),
            "Luna Star".to_string(),
            18,
            stardom_core::attribute::BaseAttributes::new(60, 55, 70, 80),
        );
        // Set interesting initial personality and image values
        sample.personality.social = 35;
        sample.personality.thinking = -20;
        sample.personality.action = 15;
        sample.personality.stance = -40;
        sample.image.pure = 45;
        sample.image.cool = 30;
        sample.image.mysterious = 20;
        sample.stats.popularity = 10;
        state.artists.push(sample);

        (Self(state), catalogs)
    }

    pub fn command(&mut self, cmd: GameCommand) {
        self.0.process_command(cmd);
    }
}
