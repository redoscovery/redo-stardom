use serde::{Serialize, de::DeserializeOwned};
use std::path::Path;

#[derive(Debug)]
pub enum SaveError {
    Serialize(ron::Error),
    Deserialize(ron::error::SpannedError),
    Io(std::io::Error),
}

impl From<ron::Error> for SaveError {
    fn from(e: ron::Error) -> Self {
        Self::Serialize(e)
    }
}

impl From<ron::error::SpannedError> for SaveError {
    fn from(e: ron::error::SpannedError) -> Self {
        Self::Deserialize(e)
    }
}

impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub fn save_to_string<T: Serialize>(state: &T) -> Result<String, SaveError> {
    Ok(ron::ser::to_string_pretty(
        state,
        ron::ser::PrettyConfig::default(),
    )?)
}

pub fn load_from_string<T: DeserializeOwned>(data: &str) -> Result<T, SaveError> {
    Ok(ron::from_str(data)?)
}

pub fn save_to_file<T: Serialize>(state: &T, path: &Path) -> Result<(), SaveError> {
    let data = save_to_string(state)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn load_from_file<T: DeserializeOwned>(path: &Path) -> Result<T, SaveError> {
    let data = std::fs::read_to_string(path)?;
    load_from_string(&data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::BaseAttributes;
    use crate::config::Settings;
    use crate::game::{GamePhase, GameState};
    use crate::types::{ArtistId, Money};

    fn make_artist(id: u32, name: &str) -> crate::artist::Artist {
        crate::artist::Artist::new(
            ArtistId(id),
            name.to_string(),
            20,
            BaseAttributes::default(),
        )
    }

    #[test]
    fn serialize_and_deserialize_game_state() {
        let game = GameState::new(Settings::default());
        let serialized = save_to_string(&game).expect("serialize");
        let loaded: GameState = load_from_string(&serialized).expect("deserialize");
        assert_eq!(loaded.calendar.year, game.calendar.year);
        assert_eq!(loaded.calendar.week, game.calendar.week);
        assert_eq!(loaded.company.balance, Money(1_000_000));
        assert_eq!(loaded.phase, GamePhase::MainGame);
    }

    #[test]
    fn roundtrip_with_artists() {
        let mut game = GameState::new(Settings::default());
        game.artists.push(make_artist(1, "Luna Star"));

        let serialized = save_to_string(&game).expect("serialize");
        let loaded: GameState = load_from_string(&serialized).expect("deserialize");
        assert_eq!(loaded.artists.len(), 1);
        assert_eq!(loaded.artists[0].name, "Luna Star");
    }

    #[test]
    fn save_to_file_and_load() {
        let mut game = GameState::new(Settings::default());
        game.artists.push(make_artist(2, "Nova"));

        let path = std::env::temp_dir().join("stardom_save_test.ron");
        save_to_file(&game, &path).expect("save to file");

        let loaded: GameState = load_from_file(&path).expect("load from file");
        assert_eq!(loaded.artists.len(), 1);
        assert_eq!(loaded.artists[0].name, "Nova");

        std::fs::remove_file(&path).ok();
    }
}
