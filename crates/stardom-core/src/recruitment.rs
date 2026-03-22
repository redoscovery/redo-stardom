use serde::{Deserialize, Serialize};

use crate::data_loader::ArtistDefinition;

const MIN_COMMISSION: f32 = 0.15;
const MAX_COMMISSION: f32 = 0.50;
const LOCKOUT_WEEKS: u32 = 26;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistProspect {
    pub definition: ArtistDefinition,
    pub location: String,
    pub available_day: u32, // 1=Mon .. 7=Sun
    pub base_commission: f32,
    pub failed_attempts: u32,
    pub locked_until_week: u32,
}

impl ArtistProspect {
    pub fn is_available(&self, day_of_week: u32, current_week: u32) -> bool {
        self.available_day == day_of_week && !self.is_locked(current_week)
    }

    pub fn is_locked(&self, current_week: u32) -> bool {
        self.failed_attempts >= 2 && current_week < self.locked_until_week
    }

    pub fn record_failure(&mut self, current_week: u32) {
        self.failed_attempts += 1;
        if self.failed_attempts >= 2 {
            self.locked_until_week = current_week + LOCKOUT_WEEKS;
        }
    }
}

pub fn negotiate_commission(base: f32, adjustment_pct: i32) -> f32 {
    (base + adjustment_pct as f32 / 100.0).clamp(MIN_COMMISSION, MAX_COMMISSION)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::{BaseAttributes, InnerTraits};
    use crate::persona::{ImageTags, PersonalitySpectrums};
    use crate::types::ArtistId;

    fn make_prospect(available_day: u32) -> ArtistProspect {
        ArtistProspect {
            definition: ArtistDefinition {
                id: ArtistId(1),
                name: "Mika".to_string(),
                starting_age: 20,
                base_attributes: BaseAttributes::default(),
                personality: PersonalitySpectrums::default(),
                traits: InnerTraits::default(),
                image: ImageTags::default(),
            },
            location: "Tokyo".to_string(),
            available_day,
            base_commission: 0.30,
            failed_attempts: 0,
            locked_until_week: 0,
        }
    }

    #[test]
    fn prospect_available_on_correct_day() {
        let prospect = make_prospect(3); // Wednesday
        assert!(prospect.is_available(3, 1));
        assert!(!prospect.is_available(2, 1));
        assert!(!prospect.is_available(4, 1));
    }

    #[test]
    fn prospect_locked_after_two_failures() {
        let mut prospect = make_prospect(1);
        prospect.record_failure(10);
        assert!(!prospect.is_locked(10)); // only 1 failure, not locked yet
        prospect.record_failure(10);
        assert!(prospect.is_locked(10)); // 2 failures → locked
        assert!(prospect.is_locked(35)); // still within lockout
        assert!(!prospect.is_locked(36)); // week 10 + 26 = 36, no longer locked
    }

    #[test]
    fn negotiate_commission_adjustment() {
        // base 0.30 - 0.05 = 0.25
        let result = negotiate_commission(0.30, -5);
        assert!((result - 0.25).abs() < 1e-6);
        // base 0.30 + 0.10 = 0.40
        let result = negotiate_commission(0.30, 10);
        assert!((result - 0.40).abs() < 1e-6);
    }

    #[test]
    fn commission_clamped_to_range() {
        // floor at MIN_COMMISSION
        let low = negotiate_commission(0.20, -20);
        assert!((low - 0.15).abs() < 1e-6);
        // cap at MAX_COMMISSION
        let high = negotiate_commission(0.40, 20);
        assert!((high - 0.50).abs() < 1e-6);
    }
}
