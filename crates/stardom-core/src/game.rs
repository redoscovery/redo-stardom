use serde::{Deserialize, Serialize};
use crate::artist::Artist;
use crate::calendar::Calendar;
use crate::company::CompanyState;
use crate::config::Settings;
use crate::types::{Activity, Money};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    MainGame,
    PostEnding,
    GameOver,
}

#[derive(Debug, Clone)]
pub enum GameCommand {
    AdvanceWeek,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub calendar: Calendar,
    pub company: CompanyState,
    pub artists: Vec<Artist>,
    pub phase: GamePhase,
    pub settings: Settings,
}

impl GameState {
    pub fn new(settings: Settings) -> Self {
        Self {
            calendar: Calendar::new(settings.goal_years),
            company: CompanyState::new(Money(settings.starting_balance), settings.max_artists),
            artists: Vec::new(),
            phase: GamePhase::MainGame,
            settings,
        }
    }

    pub fn process_command(&mut self, command: GameCommand) {
        if self.phase == GamePhase::GameOver {
            return;
        }
        match command {
            GameCommand::AdvanceWeek => self.advance_week(),
        }
    }

    fn advance_week(&mut self) {
        let was_last_week_of_year = self.calendar.week == 52;
        self.calendar.advance_week();

        // Age artists on year rollover
        if was_last_week_of_year {
            for artist in &mut self.artists {
                artist.age += 1;
            }
        }

        // Update each artist's popularity decay
        for artist in &mut self.artists {
            let active = artist.current_activity.is_public();
            artist.inactive_weeks = if active { 0 } else { artist.inactive_weeks + 1 };
            artist.stats.apply_weekly_popularity_decay(active, artist.inactive_weeks);
            // Reset activity to Idle for next week
            artist.current_activity = Activity::Idle;
        }

        // Update bankruptcy counter
        let has_pending_income = false; // TODO: check pending gig income
        self.company.update_bankruptcy_counter(has_pending_income);

        // Check phase transitions
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
    use crate::config::Settings;
    use crate::types::ArtistId;

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
}
