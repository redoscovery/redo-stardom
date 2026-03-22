use serde::{Deserialize, Serialize};

use crate::persona::ImageTag;
use crate::stats::RecognitionTier;
use crate::types::CrisisId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrisisChoice {
    pub label: String,
    pub reputation_change: i32,
    pub popularity_change: i32,
    pub stress_change: i32,
    pub image_tag_changes: Vec<(ImageTag, i32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrisisDef {
    pub id: CrisisId,
    pub name: String,
    pub description: String,
    pub trigger_weight: u32,
    pub min_recognition_tier: RecognitionTier,
    pub choices: Vec<CrisisChoice>,
}

pub struct CrisisEffect {
    pub reputation_change: i32,
    pub popularity_change: i32,
    pub stress_change: i32,
    pub image_tag_changes: Vec<(ImageTag, i32)>,
}

impl CrisisDef {
    pub fn can_trigger(&self, artist_tier: RecognitionTier) -> bool {
        artist_tier >= self.min_recognition_tier
    }

    /// Returns None if choices is empty.
    pub fn resolve(&self, choice_index: usize) -> Option<CrisisEffect> {
        let choice = self
            .choices
            .get(choice_index)
            .or_else(|| self.choices.first())?;
        Some(CrisisEffect {
            reputation_change: choice.reputation_change,
            popularity_change: choice.popularity_change,
            stress_change: choice.stress_change,
            image_tag_changes: choice.image_tag_changes.clone(),
        })
    }
}

/// roll: random value 0..100. Rebellion > 70 adds 20% to chance (spec A.6).
pub fn roll_crisis_chance(base_chance: u32, rebellion: i32, roll: u32) -> bool {
    let mut chance = base_chance;
    if rebellion > 70 {
        chance = chance * 120 / 100;
    }
    roll < chance
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_crisis() -> CrisisDef {
        CrisisDef {
            id: CrisisId(1),
            name: "Dating Scandal".to_string(),
            description: "Photos of you on a date have leaked online.".to_string(),
            trigger_weight: 10,
            min_recognition_tier: RecognitionTier::Newcomer,
            choices: vec![
                CrisisChoice {
                    label: "Deny everything".to_string(),
                    reputation_change: -5,
                    popularity_change: 5,
                    stress_change: 10,
                    image_tag_changes: vec![(ImageTag::Pure, -10)],
                },
                CrisisChoice {
                    label: "Come clean".to_string(),
                    reputation_change: 10,
                    popularity_change: -5,
                    stress_change: 5,
                    image_tag_changes: vec![(ImageTag::Pure, 5), (ImageTag::Mysterious, -5)],
                },
            ],
        }
    }

    #[test]
    fn crisis_choice_count() {
        let crisis = make_crisis();
        assert_eq!(crisis.choices.len(), 2);
    }

    #[test]
    fn apply_crisis_choice() {
        let crisis = make_crisis();
        let effect = crisis.resolve(1).unwrap();
        assert_eq!(effect.reputation_change, 10);
        assert_eq!(effect.popularity_change, -5);
        assert_eq!(effect.stress_change, 5);
        assert_eq!(effect.image_tag_changes.len(), 2);
        assert_eq!(effect.image_tag_changes[0], (ImageTag::Pure, 5));
        assert_eq!(effect.image_tag_changes[1], (ImageTag::Mysterious, -5));
    }

    #[test]
    fn crisis_resolve_out_of_bounds_defaults_to_first() {
        let crisis = make_crisis();
        let effect = crisis.resolve(99).unwrap();
        // Should fall back to first choice
        assert_eq!(effect.reputation_change, -5);
        assert_eq!(effect.popularity_change, 5);
        assert_eq!(effect.stress_change, 10);
        assert_eq!(effect.image_tag_changes.len(), 1);
        assert_eq!(effect.image_tag_changes[0], (ImageTag::Pure, -10));
    }

    #[test]
    fn crisis_resolve_empty_choices_returns_none() {
        let mut crisis = make_crisis();
        crisis.choices.clear();
        assert!(crisis.resolve(0).is_none());
    }

    #[test]
    fn should_trigger_based_on_tier() {
        let crisis = make_crisis(); // min_recognition_tier = Newcomer
        assert!(!crisis.can_trigger(RecognitionTier::Unknown));
        assert!(crisis.can_trigger(RecognitionTier::Newcomer));
        assert!(crisis.can_trigger(RecognitionTier::Rising));
        assert!(crisis.can_trigger(RecognitionTier::Star));
    }

    #[test]
    fn serialization_roundtrip() {
        let crisis = make_crisis();
        let serialized = ron::to_string(&crisis).unwrap();
        let deserialized: CrisisDef = ron::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, crisis.id);
        assert_eq!(deserialized.name, crisis.name);
        assert_eq!(deserialized.choices.len(), crisis.choices.len());
        assert_eq!(
            deserialized.choices[0].reputation_change,
            crisis.choices[0].reputation_change
        );
    }

    #[test]
    fn roll_crisis_chance_normal() {
        // base_chance=10, rebellion=50 (not > 70), roll=9 → true
        assert!(roll_crisis_chance(10, 50, 9));
        // roll=10 → false
        assert!(!roll_crisis_chance(10, 50, 10));
    }

    #[test]
    fn roll_crisis_chance_high_rebellion_boosts_chance() {
        // base_chance=10, rebellion=80 → effective chance = 12
        // roll=11 → true (would be false without boost)
        assert!(roll_crisis_chance(10, 80, 11));
        // roll=12 → false
        assert!(!roll_crisis_chance(10, 80, 12));
    }
}
