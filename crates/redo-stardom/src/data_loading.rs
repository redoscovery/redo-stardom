use bevy::prelude::*;
use stardom_core::game::GameState;
use stardom_core::job::JobDef;
use stardom_core::training::TrainingDef;

/// Catalogs that live outside GameState (training & job defs are passed inline via GameCommand).
#[derive(Resource, Default)]
pub struct GameCatalogs {
    pub training: Vec<TrainingDef>,
    pub jobs: Vec<JobDef>,
}

fn try_load<T: serde::de::DeserializeOwned>(path: &str) -> Vec<T> {
    match std::fs::read_to_string(path) {
        Ok(data) => ron::from_str(&data).unwrap_or_else(|e| {
            eprintln!("Warning: failed to parse {}: {}", path, e);
            vec![]
        }),
        Err(_) => {
            eprintln!("Warning: could not read {}", path);
            vec![]
        }
    }
}

/// Load catalog data from RON files into GameState fields.
pub fn load_game_data(game: &mut GameState) {
    game.gig_catalog = try_load("data/gigs/default_gigs.ron");
    game.office_upgrades = try_load("data/offices/default_offices.ron");
    game.outfit_catalog = try_load("data/outfits/default_outfits.ron");
    game.prospects = try_load("data/artists/default_prospects.ron");
}

/// Load training and job catalogs that are stored as a separate Bevy resource.
pub fn load_catalogs() -> GameCatalogs {
    GameCatalogs {
        training: try_load("data/training/default_training.ron"),
        jobs: try_load("data/jobs/default_jobs.ron"),
    }
}
