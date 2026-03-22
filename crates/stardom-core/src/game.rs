use crate::artist::Artist;
use crate::calendar::{Calendar, WEEKS_PER_YEAR};
use crate::company::CompanyState;
use crate::config::Settings;
use crate::crisis::CrisisDef;
use crate::gig::{GigCategory, GigDef};
use crate::job::JobDef;
use crate::office::{self, OfficeUpgradeDef};
use crate::scheduling;
use crate::training::TrainingDef;
use crate::types::{Activity, ArtistId, AwardId, Money};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    MainGame,
    PostEnding,
    GameOver,
}

#[derive(Debug, Clone)]
pub enum GameCommand {
    AdvanceWeek,
    AssignTraining {
        artist_index: usize,
        training: TrainingDef,
    },
    AssignJob {
        artist_index: usize,
        job: JobDef,
    },
    AssignGig {
        artist_index: usize,
        gig: GigDef,
    },
    AssignRest {
        artist_index: usize,
    },
    UpgradeOffice,
    DowngradeOffice,
    RespondToCrisis {
        crisis_index: usize,
        choice: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub calendar: Calendar,
    pub company: CompanyState,
    pub artists: Vec<Artist>,
    pub phase: GamePhase,
    pub settings: Settings,
    #[serde(default)]
    pub pending_gigs: Vec<(usize, GigDef)>,
    #[serde(default)]
    pub office_upgrades: Vec<OfficeUpgradeDef>,
    #[serde(default)]
    pub active_crises: Vec<(usize, CrisisDef)>,
    #[serde(default)]
    pub awards_won: Vec<(ArtistId, AwardId)>,
    #[serde(default)]
    pub completed_gig_categories: Vec<(ArtistId, GigCategory)>,
}

impl GameState {
    pub fn new(settings: Settings) -> Self {
        Self {
            calendar: Calendar::new(settings.goal_years),
            company: CompanyState::new(Money(settings.starting_balance), settings.max_artists),
            artists: Vec::new(),
            phase: GamePhase::MainGame,
            settings,
            pending_gigs: Vec::new(),
            office_upgrades: Vec::new(),
            active_crises: Vec::new(),
            awards_won: Vec::new(),
            completed_gig_categories: Vec::new(),
        }
    }

    pub fn process_command(&mut self, command: GameCommand) {
        if self.phase == GamePhase::GameOver {
            return;
        }
        match command {
            GameCommand::AdvanceWeek => self.advance_week(),
            GameCommand::AssignTraining {
                artist_index,
                training,
            } => {
                if let Some(artist) = self.artists.get_mut(artist_index)
                    && !artist.is_locked()
                    && artist.current_activity == Activity::Idle
                {
                    let cost = scheduling::apply_training(artist, &training);
                    self.company.spend(Money(cost));
                }
            }
            GameCommand::AssignJob { artist_index, job } => {
                if let Some(artist) = self.artists.get_mut(artist_index)
                    && !artist.is_locked()
                    && artist.current_activity == Activity::Idle
                {
                    let pay = scheduling::apply_job(artist, &job);
                    self.company.earn(Money(pay));
                }
            }
            GameCommand::AssignGig { artist_index, gig } => {
                if let Some(artist) = self.artists.get_mut(artist_index)
                    && !artist.is_locked()
                    && artist.current_activity == Activity::Idle
                {
                    scheduling::start_gig(artist, &gig);
                    self.pending_gigs.push((artist_index, gig));
                }
            }
            GameCommand::AssignRest { artist_index } => {
                if let Some(artist) = self.artists.get_mut(artist_index)
                    && !artist.is_locked()
                    && artist.current_activity == Activity::Idle
                {
                    scheduling::apply_rest(artist);
                }
            }
            GameCommand::UpgradeOffice => {
                if let Some(upgrade) = office::next_upgrade(self.company.office_tier, &self.office_upgrades) {
                    if office::can_afford(self.company.balance, upgrade) {
                        let cost = upgrade.cost;
                        let tier = upgrade.tier;
                        self.company.spend(cost);
                        self.company.office_tier = tier;
                    }
                }
            }
            GameCommand::DowngradeOffice => {
                let refund = office::downgrade_refund(self.company.office_tier, &self.office_upgrades);
                if refund.0 > 0 {
                    self.company.earn(refund);
                    // Find previous tier: the tier just before current in upgrades list
                    let current = self.company.office_tier;
                    let prev_tier = self.office_upgrades
                        .iter()
                        .filter(|u| u.tier < current)
                        .last()
                        .map(|u| u.tier)
                        .unwrap_or(crate::company::OfficeTier::Starter);
                    self.company.office_tier = prev_tier;
                }
            }
            GameCommand::RespondToCrisis { crisis_index, choice } => {
                if crisis_index < self.active_crises.len() {
                    let (artist_idx, crisis) = self.active_crises.remove(crisis_index);
                    if let Some(effect) = crisis.resolve(choice) {
                        if let Some(artist) = self.artists.get_mut(artist_idx) {
                            artist.stats.reputation += effect.reputation_change;
                            artist.stats.popularity += effect.popularity_change;
                            artist.stats.stress += effect.stress_change;
                            for (tag, delta) in &effect.image_tag_changes {
                                match tag {
                                    crate::persona::ImageTag::Pure => artist.image.pure += delta,
                                    crate::persona::ImageTag::Sexy => artist.image.sexy += delta,
                                    crate::persona::ImageTag::Cool => artist.image.cool += delta,
                                    crate::persona::ImageTag::Intellectual => artist.image.intellectual += delta,
                                    crate::persona::ImageTag::Funny => artist.image.funny += delta,
                                    crate::persona::ImageTag::Mysterious => artist.image.mysterious += delta,
                                }
                            }
                            artist.image.clamp();
                        }
                    }
                }
            }
        }
    }

    fn advance_week(&mut self) {
        let was_last_week_of_year = self.calendar.week == WEEKS_PER_YEAR;
        self.calendar.advance_week();

        // Decrement gig lock timers
        for artist in &mut self.artists {
            if artist.locked_weeks > 0 {
                artist.locked_weeks -= 1;
            }
        }

        // Complete finished gigs (locked_weeks reached 0)
        if !self.pending_gigs.is_empty() {
            let mut pending = std::mem::take(&mut self.pending_gigs);
            pending.retain(|(idx, gig_def)| {
                let is_complete = self.artists.get(*idx).is_some_and(|a| a.locked_weeks == 0);
                if is_complete {
                    let artist = &mut self.artists[*idx];
                    let pay = scheduling::complete_gig(artist, gig_def);
                    self.company.earn(Money(pay));
                    // Track completed gig category
                    self.completed_gig_categories.push((artist.id, gig_def.category));
                    false
                } else {
                    true
                }
            });
            self.pending_gigs = pending;
        }

        // Aging, popularity decay, activity reset
        for artist in &mut self.artists {
            if was_last_week_of_year {
                artist.age += 1;
            }
            let active = artist.current_activity.is_public();
            artist.inactive_weeks = if active { 0 } else { artist.inactive_weeks + 1 };
            artist
                .stats
                .apply_weekly_popularity_decay(active, artist.inactive_weeks);
            if !artist.is_locked() {
                artist.current_activity = Activity::Idle;
            }
        }

        // Deduct weekly office upkeep
        let upkeep = office::get_weekly_upkeep(self.company.office_tier, &self.office_upgrades);
        if upkeep.0 > 0 {
            self.company.spend(upkeep);
        }

        // Bankruptcy — pending gigs count as pending income
        let has_pending_income = !self.pending_gigs.is_empty();
        self.company.update_bankruptcy_counter(has_pending_income);

        if self.company.is_bankrupt() {
            self.phase = GamePhase::GameOver;
        } else if self.phase == GamePhase::MainGame && self.calendar.is_goal_period_over() {
            self.phase = GamePhase::PostEnding;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::BaseAttributes;
    use crate::company::OfficeTier;
    use crate::config::Settings;
    use crate::crisis::{CrisisChoice, CrisisDef};
    use crate::gig::{GigCategory, GigDef};
    use crate::job::JobDef;
    use crate::office::OfficeUpgradeDef;
    use crate::persona::ImageTag;
    use crate::stats::RecognitionTier;
    use crate::training::{PrimaryAttribute, SkillTarget, TrainingDef, TrainingTier};
    use crate::types::{ArtistId, CrisisId, GigId, JobId, TrainingId};

    fn default_game() -> GameState {
        GameState::new(Settings::default())
    }

    fn make_artist_with_popularity(pop: i32) -> Artist {
        let mut artist = Artist::new(
            ArtistId(1),
            "Test".to_string(),
            20,
            BaseAttributes::default(),
        );
        artist.stats.popularity = pop;
        artist
    }

    fn sample_training() -> TrainingDef {
        TrainingDef {
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
        }
    }

    fn sample_job() -> JobDef {
        JobDef {
            id: JobId(1),
            name: "Street".to_string(),
            pay: 600,
            skill_gains: vec![(SkillTarget::Vocal, 15)],
            skill_losses: vec![],
            recognition_gain: 3,
            stress_change: 3,
            required_recognition_tier: RecognitionTier::Unknown,
        }
    }

    fn sample_gig() -> GigDef {
        GigDef {
            id: GigId(1),
            name: "Single".to_string(),
            category: GigCategory::Music,
            duration_weeks: 2,
            required_recognition_tier: RecognitionTier::Unknown,
            skill_weights: vec![(SkillTarget::Vocal, 1.0)],
            base_pay: 50_000,
            recognition_reward: 50,
            reputation_reward: 3,
            stress_cost: 10,
            ideal_image_tags: vec![],
            conflicting_image_tags: vec![],
            personality_preference: None,
            skill_gains: vec![(SkillTarget::Vocal, 30)],
        }
    }

    #[test]
    fn new_game_state() {
        let game = default_game();
        assert_eq!(game.calendar.year, 1);
        assert_eq!(game.calendar.week, 1);
        assert_eq!(game.company.balance, Money(1_000_000));
        assert!(game.artists.is_empty());
        assert_eq!(game.phase, GamePhase::MainGame);
    }

    #[test]
    fn advance_week_updates_calendar() {
        let mut game = default_game();
        game.process_command(GameCommand::AdvanceWeek);
        assert_eq!(game.calendar.week, 2);
    }

    #[test]
    fn advance_week_decays_popularity() {
        let mut game = default_game();
        game.artists.push(make_artist_with_popularity(50));
        game.process_command(GameCommand::AdvanceWeek);
        // inactive_weeks becomes 1, apply_weekly_popularity_decay(false, 1)
        // base_decay=2, inactivity_penalty=2 (1 consecutive week) → 50-4=46
        assert_eq!(game.artists[0].stats.popularity, 46);
    }

    #[test]
    fn artists_age_on_year_rollover() {
        let mut game = default_game();
        game.artists.push(make_artist_with_popularity(0));
        // Advance to the last week of year 1 (week 52 is week 52)
        // Start at week 1; need to advance 51 times to reach week 52, then one more to trigger rollover
        for _ in 0..52 {
            game.process_command(GameCommand::AdvanceWeek);
        }
        assert_eq!(game.artists[0].age, 21);
    }

    #[test]
    fn game_phase_transitions() {
        let mut game = default_game(); // goal_years = 3
        for _ in 0..156 {
            game.process_command(GameCommand::AdvanceWeek);
        }
        assert_eq!(game.phase, GamePhase::PostEnding);
    }

    #[test]
    fn bankruptcy_ends_game() {
        let settings = Settings {
            starting_balance: -1,
            ..Settings::default()
        };
        let mut game = GameState::new(settings);
        // balance is -1 from the start; each week increments consecutive_negative_weeks
        // after 4 weeks: counter=4, is_bankrupt() → true → GameOver
        for _ in 0..4 {
            game.process_command(GameCommand::AdvanceWeek);
        }
        assert_eq!(game.phase, GamePhase::GameOver);
    }

    #[test]
    fn assign_training_deducts_cost() {
        let mut game = default_game();
        game.artists.push(make_artist_with_popularity(0));
        game.process_command(GameCommand::AssignTraining {
            artist_index: 0,
            training: sample_training(),
        });
        game.process_command(GameCommand::AdvanceWeek);
        assert_eq!(game.company.balance, Money(1_000_000 - 8_000));
        assert_eq!(game.artists[0].skills.vocal, 40);
    }

    #[test]
    fn assign_job_earns_money() {
        let mut game = default_game();
        game.artists.push(make_artist_with_popularity(0));
        game.process_command(GameCommand::AssignJob {
            artist_index: 0,
            job: sample_job(),
        });
        game.process_command(GameCommand::AdvanceWeek);
        assert_eq!(game.company.balance, Money(1_000_000 + 600));
        assert_eq!(game.artists[0].skills.vocal, 15);
    }

    #[test]
    fn assign_gig_locks_and_completes() {
        let mut game = default_game();
        game.artists.push(make_artist_with_popularity(50));
        game.process_command(GameCommand::AssignGig {
            artist_index: 0,
            gig: sample_gig(),
        });

        // Week 1: locked_weeks goes from 2 to 1
        game.process_command(GameCommand::AdvanceWeek);
        assert_eq!(game.artists[0].locked_weeks, 1);
        assert_eq!(game.artists[0].current_activity, Activity::Gig);
        // Popularity: 50, gig is public so no inactivity penalty, base_decay -2 → 48

        // Week 2: gig completes (locked_weeks 1→0), rewards applied
        game.process_command(GameCommand::AdvanceWeek);
        assert_eq!(game.artists[0].locked_weeks, 0);
        assert_eq!(game.artists[0].skills.vocal, 30);
        // Pay: base 50000, popularity was 48 at completion → modifier 1.0+(48-50)/200=0.99 → 49500
        assert_eq!(game.company.balance, Money(1_000_000 + 49_500));
    }

    #[test]
    fn locked_artist_cannot_be_reassigned() {
        let mut game = default_game();
        game.artists.push(make_artist_with_popularity(0));
        game.process_command(GameCommand::AssignGig {
            artist_index: 0,
            gig: sample_gig(),
        });
        game.process_command(GameCommand::AdvanceWeek);
        assert!(game.artists[0].is_locked());
        // Try training while locked — should be ignored
        game.process_command(GameCommand::AssignTraining {
            artist_index: 0,
            training: sample_training(),
        });
        game.process_command(GameCommand::AdvanceWeek);
        // Only gig rewards (30), no training gains
        assert_eq!(game.artists[0].skills.vocal, 30);
    }

    #[test]
    fn rest_reduces_stress() {
        let mut game = default_game();
        let mut artist = make_artist_with_popularity(0);
        artist.stats.stress = 40;
        game.artists.push(artist);
        game.process_command(GameCommand::AssignRest { artist_index: 0 });
        game.process_command(GameCommand::AdvanceWeek);
        assert_eq!(game.artists[0].stats.stress, 20);
    }

    fn game_with_office_data() -> GameState {
        let mut game = default_game();
        game.office_upgrades = vec![
            OfficeUpgradeDef {
                tier: OfficeTier::Standard,
                cost: Money(500_000),
                max_artists_bonus: 1,
                training_cost_discount_pct: 5,
                weekly_upkeep: Money(2_000),
            },
            OfficeUpgradeDef {
                tier: OfficeTier::Premium,
                cost: Money(2_000_000),
                max_artists_bonus: 2,
                training_cost_discount_pct: 10,
                weekly_upkeep: Money(5_000),
            },
            OfficeUpgradeDef {
                tier: OfficeTier::Luxury,
                cost: Money(5_000_000),
                max_artists_bonus: 3,
                training_cost_discount_pct: 15,
                weekly_upkeep: Money(10_000),
            },
        ];
        game
    }

    #[test]
    fn office_upgrade_deducts_cost_and_increases_tier() {
        let mut game = game_with_office_data();
        assert_eq!(game.company.balance, Money(1_000_000));
        game.process_command(GameCommand::UpgradeOffice);
        assert_eq!(game.company.office_tier, OfficeTier::Standard);
        assert_eq!(game.company.balance, Money(500_000));
    }

    #[test]
    fn office_downgrade_refunds_40_pct() {
        let mut game = game_with_office_data();
        // Upgrade to Standard: 1M - 500K = 500K
        game.process_command(GameCommand::UpgradeOffice);
        assert_eq!(game.company.balance, Money(500_000));
        // Downgrade: refund 40% of 500K = 200K → 500K + 200K = 700K
        game.process_command(GameCommand::DowngradeOffice);
        assert_eq!(game.company.office_tier, OfficeTier::Starter);
        assert_eq!(game.company.balance, Money(700_000));
    }

    #[test]
    fn weekly_upkeep_deducted() {
        let mut game = game_with_office_data();
        game.process_command(GameCommand::UpgradeOffice); // Standard, upkeep 2000
        let balance_after_upgrade = game.company.balance;
        game.process_command(GameCommand::AdvanceWeek);
        assert_eq!(
            game.company.balance,
            balance_after_upgrade - Money(2_000)
        );
    }

    #[test]
    fn crisis_respond_applies_effects() {
        let mut game = default_game();
        game.artists.push(make_artist_with_popularity(50));
        let crisis = CrisisDef {
            id: CrisisId(1),
            name: "Scandal".to_string(),
            description: "A scandal has erupted.".to_string(),
            trigger_weight: 10,
            min_recognition_tier: RecognitionTier::Unknown,
            choices: vec![
                CrisisChoice {
                    label: "Deny".to_string(),
                    reputation_change: -5,
                    popularity_change: 10,
                    stress_change: 8,
                    image_tag_changes: vec![(ImageTag::Pure, -15)],
                },
                CrisisChoice {
                    label: "Apologize".to_string(),
                    reputation_change: 5,
                    popularity_change: -3,
                    stress_change: 3,
                    image_tag_changes: vec![],
                },
            ],
        };
        game.active_crises.push((0, crisis));

        let rep_before = game.artists[0].stats.reputation;
        let pop_before = game.artists[0].stats.popularity;
        let stress_before = game.artists[0].stats.stress;

        game.process_command(GameCommand::RespondToCrisis {
            crisis_index: 0,
            choice: 0,
        });

        assert_eq!(game.artists[0].stats.reputation, rep_before - 5);
        assert_eq!(game.artists[0].stats.popularity, pop_before + 10);
        assert_eq!(game.artists[0].stats.stress, stress_before + 8);
        assert!(game.active_crises.is_empty());
    }
}
