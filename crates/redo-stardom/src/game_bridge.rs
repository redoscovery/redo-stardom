use bevy::prelude::*;
use stardom_core::config::Settings;
use stardom_core::game::{GameCommand, GameState};

#[derive(Resource)]
pub struct GameWorld(pub GameState);

impl GameWorld {
    pub fn new_game() -> Self {
        Self(GameState::new(Settings::default()))
    }

    pub fn command(&mut self, cmd: GameCommand) {
        self.0.process_command(cmd);
    }
}
