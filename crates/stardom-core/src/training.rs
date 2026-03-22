use serde::{Deserialize, Serialize};

use crate::attribute::BaseAttributes;
use crate::stats::stress_condition_modifier;
use crate::types::TrainingId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillTarget {
    Vocal,
    Acting,
    Dance,
    Poise,
    Eloquence,
    Creativity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimaryAttribute {
    Stamina,
    Intellect,
    Empathy,
    Charm,
}

impl PrimaryAttribute {
    pub fn value_from(&self, attrs: &BaseAttributes) -> i32 {
        match self {
            Self::Stamina => attrs.stamina,
            Self::Intellect => attrs.intellect,
            Self::Empathy => attrs.empathy,
            Self::Charm => attrs.charm,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingTier {
    pub cost: i64,
    pub base_gain: i32,
    pub stress_increase: i32,
    pub unlock_threshold: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingDef {
    pub id: TrainingId,
    pub name: String,
    pub skill: SkillTarget,
    pub tiers: Vec<TrainingTier>,
    pub primary_attribute: PrimaryAttribute,
    pub secondary_attribute: Option<PrimaryAttribute>,
}

pub struct TrainingEffect {
    pub skill_target: SkillTarget,
    pub skill_gain: i32,
    pub stress_increase: i32,
    pub cost: i64,
}

impl TrainingDef {
    pub fn best_tier_index(&self, current_skill: i32) -> usize {
        let mut best = 0;
        for (i, tier) in self.tiers.iter().enumerate() {
            if current_skill >= tier.unlock_threshold {
                best = i;
            }
        }
        best
    }

    /// Spec A.1: effective_gain = base_gain * (1.0 + primary_bonus + secondary_bonus) * condition_modifier
    pub fn calculate_effect(
        &self,
        tier_index: usize,
        base_attrs: &BaseAttributes,
        stress: i32,
    ) -> TrainingEffect {
        let tier = &self.tiers[tier_index];
        let primary_bonus = (self.primary_attribute.value_from(base_attrs) - 50) as f64 / 100.0;
        let secondary_bonus = self
            .secondary_attribute
            .map(|a| (a.value_from(base_attrs) - 50) as f64 / 200.0)
            .unwrap_or(0.0);
        let attr_bonus = 1.0 + primary_bonus + secondary_bonus;
        let condition = stress_condition_modifier(stress);
        let effective_gain = (tier.base_gain as f64 * attr_bonus * condition) as i32;
        TrainingEffect {
            skill_target: self.skill,
            skill_gain: effective_gain,
            stress_increase: tier.stress_increase,
            cost: tier.cost,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vocal_training() -> TrainingDef {
        TrainingDef {
            id: TrainingId(1),
            name: "Vocal Training".to_string(),
            skill: SkillTarget::Vocal,
            tiers: vec![
                TrainingTier {
                    cost: 8000,
                    base_gain: 40,
                    stress_increase: 5,
                    unlock_threshold: 0,
                },
                TrainingTier {
                    cost: 15000,
                    base_gain: 60,
                    stress_increase: 8,
                    unlock_threshold: 1000,
                },
                TrainingTier {
                    cost: 25000,
                    base_gain: 80,
                    stress_increase: 12,
                    unlock_threshold: 3000,
                },
                TrainingTier {
                    cost: 40000,
                    base_gain: 100,
                    stress_increase: 15,
                    unlock_threshold: 6500,
                },
            ],
            primary_attribute: PrimaryAttribute::Empathy,
            secondary_attribute: Some(PrimaryAttribute::Charm),
        }
    }

    #[test]
    fn best_available_tier() {
        let training = make_vocal_training();
        assert_eq!(training.best_tier_index(0), 0);
        assert_eq!(training.best_tier_index(999), 0);
        assert_eq!(training.best_tier_index(1000), 1);
        assert_eq!(training.best_tier_index(6500), 3);
    }

    #[test]
    fn training_effect_basic() {
        // attrs all 50 → primary_bonus = 0, secondary_bonus = 0, attr_bonus = 1.0
        // stress 0 → condition = 1.0
        // gain = 40 * 1.0 * 1.0 = 40
        let training = make_vocal_training();
        let attrs = BaseAttributes::default(); // all 50
        let effect = training.calculate_effect(0, &attrs, 0);
        assert_eq!(effect.skill_gain, 40);
        assert_eq!(effect.stress_increase, 5);
        assert_eq!(effect.cost, 8000);
        assert_eq!(effect.skill_target, SkillTarget::Vocal);
    }

    #[test]
    fn training_effect_with_high_attribute() {
        // empathy=80 primary: (80-50)/100 = 0.30
        // charm=70 secondary: (70-50)/200 = 0.10
        // attr_bonus = 1.0 + 0.30 + 0.10 = 1.40
        // stress 0 → condition = 1.0
        // gain = 40 * 1.40 * 1.0 = 56.0 → 56
        let training = make_vocal_training();
        let attrs = BaseAttributes::new(50, 50, 80, 70);
        let effect = training.calculate_effect(0, &attrs, 0);
        assert_eq!(effect.skill_gain, 56);
    }

    #[test]
    fn training_effect_under_stress() {
        // attrs all 50 → attr_bonus = 1.0
        // stress=45 → condition = 0.85
        // gain = 40 * 1.0 * 0.85 = 34.0 → 34
        let training = make_vocal_training();
        let attrs = BaseAttributes::default();
        let effect = training.calculate_effect(0, &attrs, 45);
        assert_eq!(effect.skill_gain, 34);
    }

    #[test]
    fn serialization_roundtrip() {
        let training = make_vocal_training();
        let serialized = ron::to_string(&training).unwrap();
        let deserialized: TrainingDef = ron::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, training.id);
        assert_eq!(deserialized.name, training.name);
        assert_eq!(deserialized.skill, training.skill);
        assert_eq!(deserialized.tiers.len(), training.tiers.len());
        assert_eq!(deserialized.primary_attribute, training.primary_attribute);
        assert_eq!(deserialized.secondary_attribute, training.secondary_attribute);
    }
}
