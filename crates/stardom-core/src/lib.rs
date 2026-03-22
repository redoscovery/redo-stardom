pub mod artist;
pub mod attribute;
pub mod award;
pub mod calendar;
pub mod company;
pub mod config;
pub mod crisis;
pub mod data_loader;
pub mod game;
pub mod gig;
pub mod gig_pool;
pub mod job;
pub mod office;
pub mod persona;
pub mod scheduling;
pub mod stats;
pub mod training;
pub mod types;

#[cfg(test)]
mod integration_tests {
    use crate::config::Settings;
    use crate::data_loader::load_artist_definition;
    use crate::game::{GameCommand, GamePhase, GameState};
    use crate::job::JobDef;
    use crate::stats::RecognitionTier;
    use crate::training::{PrimaryAttribute, SkillTarget, TrainingDef, TrainingTier};
    use crate::types::{JobId, Money, TrainingId};

    #[test]
    fn full_game_loop_with_activities() {
        let mut game = GameState::new(Settings::default());

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

        let training = TrainingDef {
            id: TrainingId(1),
            name: "Vocal".to_string(),
            skill: SkillTarget::Vocal,
            tiers: vec![TrainingTier {
                cost: 8_000,
                base_gain: 40,
                stress_increase: 5,
                unlock_threshold: 0,
            }],
            primary_attribute: PrimaryAttribute::Empathy,
            secondary_attribute: None,
        };

        let job = JobDef {
            id: JobId(1),
            name: "Street".to_string(),
            pay: 1_000,
            skill_gains: vec![(SkillTarget::Vocal, 10)],
            skill_losses: vec![],
            recognition_gain: 5,
            stress_change: 3,
            required_recognition_tier: RecognitionTier::Unknown,
        };

        // Alternate training, jobs, and rest for 10 weeks
        for i in 0..10 {
            if i % 3 == 2 {
                game.process_command(GameCommand::AssignRest { artist_index: 0 });
            } else if i % 3 == 0 {
                game.process_command(GameCommand::AssignTraining {
                    artist_index: 0,
                    training: training.clone(),
                });
            } else {
                game.process_command(GameCommand::AssignJob {
                    artist_index: 0,
                    job: job.clone(),
                });
            }
            game.process_command(GameCommand::AdvanceWeek);
        }

        // Artist should have gained skills
        assert!(game.artists[0].skills.vocal > 0);
        // Company balance should have changed (training costs - job income)
        assert_ne!(game.company.balance, Money(1_000_000));
        // Stress should be managed due to rest weeks
        assert!(game.artists[0].stats.stress < 50);
        // Game should still be in main phase
        assert_eq!(game.phase, GamePhase::MainGame);
    }
}
