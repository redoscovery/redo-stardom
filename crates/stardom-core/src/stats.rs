use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecognitionTier {
    Unknown,     // 0–99
    Newcomer,    // 100–499
    Rising,      // 500–1,999
    Established, // 2,000–4,999
    Star,        // 5,000–14,999
    Superstar,   // 15,000+
}

impl RecognitionTier {
    pub fn from_value(recognition: i64) -> Self {
        match recognition {
            0..100 => Self::Unknown,
            100..500 => Self::Newcomer,
            500..2_000 => Self::Rising,
            2_000..5_000 => Self::Established,
            5_000..15_000 => Self::Star,
            _ => Self::Superstar,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AuxiliaryStats {
    pub recognition: i64, // 0 → ∞, only increases
    pub reputation: i32,  // -100 to +100
    pub popularity: i32,  // 0 to 100, decays weekly
    pub stress: i32,      // 0 to 100
}

impl AuxiliaryStats {
    pub fn add_recognition(&mut self, amount: i64) {
        if amount > 0 {
            self.recognition += amount;
        }
    }

    pub fn recognition_tier(&self) -> RecognitionTier {
        RecognitionTier::from_value(self.recognition)
    }

    pub fn clamp(&mut self) {
        self.reputation = self.reputation.clamp(-100, 100);
        self.popularity = self.popularity.clamp(0, 100);
        self.stress = self.stress.clamp(0, 100);
        if self.recognition < 0 {
            self.recognition = 0;
        }
    }

    /// Apply weekly popularity decay.
    pub fn apply_weekly_popularity_decay(
        &mut self,
        active_this_week: bool,
        consecutive_inactive_weeks: u32,
    ) {
        let base_decay = 2;
        let inactivity_penalty = if active_this_week {
            0
        } else {
            match consecutive_inactive_weeks {
                0 => 0,
                1 => 2,
                2 => 4,
                _ => 6,
            }
        };
        self.popularity = (self.popularity - base_decay - inactivity_penalty).max(0);
    }
}

/// Returns training/activity efficiency modifier based on stress level.
/// Spec A.4: 0-30 → 1.0, 31-60 → 0.85, 61-80 → 0.65, 81-100 → 0.40
pub fn stress_condition_modifier(stress: i32) -> f64 {
    match stress {
        0..=30 => 1.0,
        31..=60 => 0.85,
        61..=80 => 0.65,
        _ => 0.40,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognition_only_increases() {
        let mut stats = AuxiliaryStats::default();
        stats.add_recognition(100);
        assert_eq!(stats.recognition, 100);
        stats.add_recognition(-50);
        assert_eq!(stats.recognition, 100);
    }

    #[test]
    fn recognition_tier_lookup() {
        assert_eq!(RecognitionTier::from_value(0), RecognitionTier::Unknown);
        assert_eq!(RecognitionTier::from_value(99), RecognitionTier::Unknown);
        assert_eq!(RecognitionTier::from_value(100), RecognitionTier::Newcomer);
        assert_eq!(RecognitionTier::from_value(499), RecognitionTier::Newcomer);
        assert_eq!(RecognitionTier::from_value(500), RecognitionTier::Rising);
        assert_eq!(RecognitionTier::from_value(1999), RecognitionTier::Rising);
        assert_eq!(
            RecognitionTier::from_value(2000),
            RecognitionTier::Established
        );
        assert_eq!(
            RecognitionTier::from_value(4999),
            RecognitionTier::Established
        );
        assert_eq!(RecognitionTier::from_value(5000), RecognitionTier::Star);
        assert_eq!(RecognitionTier::from_value(14999), RecognitionTier::Star);
        assert_eq!(
            RecognitionTier::from_value(15000),
            RecognitionTier::Superstar
        );
        assert_eq!(
            RecognitionTier::from_value(99999),
            RecognitionTier::Superstar
        );
    }

    #[test]
    fn reputation_clamp() {
        let mut stats = AuxiliaryStats {
            reputation: 150,
            ..Default::default()
        };
        stats.clamp();
        assert_eq!(stats.reputation, 100);

        stats.reputation = -150;
        stats.clamp();
        assert_eq!(stats.reputation, -100);
    }

    #[test]
    fn popularity_decay_active() {
        let mut stats = AuxiliaryStats {
            popularity: 50,
            ..Default::default()
        };
        stats.apply_weekly_popularity_decay(true, 0);
        assert_eq!(stats.popularity, 48);
    }

    #[test]
    fn popularity_decay_inactive_3_weeks() {
        let mut stats = AuxiliaryStats {
            popularity: 50,
            ..Default::default()
        };
        // inactive, 3 consecutive inactive weeks → base_decay=2, inactivity_penalty=6 → 50-8=42
        stats.apply_weekly_popularity_decay(false, 3);
        assert_eq!(stats.popularity, 42);
    }

    #[test]
    fn popularity_floor_at_zero() {
        let mut stats = AuxiliaryStats {
            popularity: 3,
            ..Default::default()
        };
        // inactive, 5 consecutive weeks → base_decay=2, inactivity_penalty=6 → 3-8=-5, clamped to 0
        stats.apply_weekly_popularity_decay(false, 5);
        assert_eq!(stats.popularity, 0);
    }

    #[test]
    fn stress_condition_modifier_brackets() {
        assert_eq!(stress_condition_modifier(0), 1.0);
        assert_eq!(stress_condition_modifier(30), 1.0);
        assert_eq!(stress_condition_modifier(31), 0.85);
        assert_eq!(stress_condition_modifier(60), 0.85);
        assert_eq!(stress_condition_modifier(61), 0.65);
        assert_eq!(stress_condition_modifier(80), 0.65);
        assert_eq!(stress_condition_modifier(81), 0.40);
        assert_eq!(stress_condition_modifier(100), 0.40);
    }
}
