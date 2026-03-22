pub mod artist;
pub mod attribute;
pub mod calendar;
pub mod company;
pub mod config;
pub mod data_loader;
pub mod game;
pub mod gig;
pub mod persona;
pub mod job;
pub mod scheduling;
pub mod stats;
pub mod training;
pub mod types;

#[cfg(test)]
mod integration_tests {
    use crate::config::Settings;
    use crate::data_loader::load_artist_definition;
    use crate::game::{GameCommand, GamePhase, GameState};

    #[test]
    fn full_game_loop_smoke_test() {
        let settings = Settings::default();
        let mut game = GameState::new(settings);
        assert_eq!(game.phase, GamePhase::MainGame);

        // Load and add an artist
        let ron_str = r#"
            ArtistDefinition(
                id: ArtistId(1),
                name: "Luna Star",
                starting_age: 18,
                base_attributes: BaseAttributes(stamina: 60, intellect: 55, empathy: 70, charm: 80),
                personality: PersonalitySpectrums(social: 30, thinking: -20, action: 10, stance: -40),
                traits: InnerTraits(confidence: 55, rebellion: 25),
                image: ImageTags(pure: 60, sexy: 20, cool: 40, intellectual: 30, funny: 10, mysterious: 50),
            )
        "#;
        let def = load_artist_definition(ron_str).unwrap();
        game.artists.push(def.into_artist());
        assert_eq!(game.artists.len(), 1);
        assert_eq!(game.artists[0].name, "Luna Star");

        // Advance a full year
        for _ in 0..52 {
            game.process_command(GameCommand::AdvanceWeek);
        }
        assert_eq!(game.calendar.year, 2);
        assert_eq!(game.phase, GamePhase::MainGame);

        // Advance to end of goal period
        for _ in 0..(52 * 2) {
            game.process_command(GameCommand::AdvanceWeek);
        }
        assert_eq!(game.calendar.year, 4);
        assert_eq!(game.phase, GamePhase::PostEnding);

        // Popularity should have decayed to 0 (no activity)
        assert_eq!(game.artists[0].stats.popularity, 0);
    }
}
