use serde::{Deserialize, Serialize};

use crate::attribute::ProfessionalSkills;
use crate::persona::{ImageTag, ImageTags, PersonalitySpectrums, Spectrum};
use crate::stats::RecognitionTier;
use crate::training::SkillTarget;
use crate::types::GigId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GigCategory {
    Music,
    FilmTv,
    Modeling,
    Variety,
    Endorsement,
    Creative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GigDef {
    pub id: GigId,
    pub name: String,
    pub category: GigCategory,
    pub duration_weeks: u32,
    pub required_recognition_tier: RecognitionTier,
    pub skill_weights: Vec<(SkillTarget, f64)>,
    pub base_pay: i64,
    pub recognition_reward: i64,
    pub reputation_reward: i32,
    pub stress_cost: i32,
    pub ideal_image_tags: Vec<(ImageTag, i32)>,
    pub conflicting_image_tags: Vec<(ImageTag, i32)>,
    pub personality_preference: Option<(Spectrum, i32)>,
    pub skill_gains: Vec<(SkillTarget, i32)>,
}

fn popularity_modifier(popularity: i32) -> f64 {
    1.0 + (popularity as f64 - 50.0) / 200.0
}

impl GigDef {
    pub fn is_available(&self, artist_tier: RecognitionTier) -> bool {
        artist_tier >= self.required_recognition_tier
    }

    /// Spec A.5 success score formula.
    pub fn calculate_success_score(
        &self,
        skills: &ProfessionalSkills,
        image: &ImageTags,
        personality: &PersonalitySpectrums,
        popularity: i32,
    ) -> i32 {
        // Weighted skill sum
        let weighted_sum: f64 = self
            .skill_weights
            .iter()
            .map(|(target, weight)| skills.get(*target) as f64 * weight)
            .sum();

        // Image modifier
        let mut image_mod: f64 = 1.0;
        for (tag, threshold) in &self.ideal_image_tags {
            let val = tag.value_from(image);
            if val >= threshold * 2 {
                image_mod += 0.10;
            } else if val >= *threshold {
                image_mod += 0.05;
            }
        }
        for (tag, threshold) in &self.conflicting_image_tags {
            let val = tag.value_from(image);
            if val > *threshold {
                image_mod -= 0.10;
            }
        }
        image_mod = image_mod.clamp(0.80, 1.20);

        // Personality modifier
        let personality_mod: f64 = if let Some((spectrum, preference)) = self.personality_preference
        {
            let artist_val = personality.get(spectrum);
            // Scale ±0.15 based on match/mismatch
            // If preference > 0, high artist_val → bonus; preference < 0, low artist_val → bonus
            let alignment = (artist_val as f64 * preference as f64) / (100.0 * 100.0);
            1.0 + alignment * 0.15
        } else {
            1.0
        };

        let pop_mod = popularity_modifier(popularity);

        (weighted_sum * image_mod * personality_mod * pop_mod) as i32
    }

    pub fn calculate_pay(&self, popularity: i32) -> i64 {
        (self.base_pay as f64 * popularity_modifier(popularity)) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::ProfessionalSkills;
    use crate::persona::{ImageTags, PersonalitySpectrums};
    use crate::stats::RecognitionTier;
    use crate::training::SkillTarget;
    use crate::types::GigId;

    fn make_music_gig() -> GigDef {
        GigDef {
            id: GigId(1),
            name: "Local Concert".to_string(),
            category: GigCategory::Music,
            duration_weeks: 2,
            required_recognition_tier: RecognitionTier::Newcomer,
            skill_weights: vec![
                (SkillTarget::Vocal, 0.5),
                (SkillTarget::Dance, 0.3),
                (SkillTarget::Poise, 0.2),
            ],
            base_pay: 50_000,
            recognition_reward: 30,
            reputation_reward: 2,
            stress_cost: 10,
            ideal_image_tags: vec![(ImageTag::Pure, 40)],
            conflicting_image_tags: vec![],
            personality_preference: None,
            skill_gains: vec![(SkillTarget::Vocal, 20), (SkillTarget::Dance, 10)],
        }
    }

    #[test]
    fn gig_availability() {
        let gig = make_music_gig();
        // Unknown tier cannot access Newcomer gig
        assert!(!gig.is_available(RecognitionTier::Unknown));
        // Newcomer and above can
        assert!(gig.is_available(RecognitionTier::Newcomer));
        assert!(gig.is_available(RecognitionTier::Rising));
        assert!(gig.is_available(RecognitionTier::Star));
    }

    #[test]
    fn gig_success_score_basic() {
        // weights 0.6/0.3/0.1 → weighted_sum = 1800+600+100 = 2500
        // pure=50 >= threshold 40 → image_mod = 1.05
        // score = 2500 * 1.05 * 1.0 * 1.0 = 2625
        let gig = GigDef {
            id: GigId(1),
            name: "Local Concert".to_string(),
            category: GigCategory::Music,
            duration_weeks: 2,
            required_recognition_tier: RecognitionTier::Newcomer,
            skill_weights: vec![
                (SkillTarget::Vocal, 0.6),
                (SkillTarget::Dance, 0.3),
                (SkillTarget::Poise, 0.1),
            ],
            base_pay: 50_000,
            recognition_reward: 30,
            reputation_reward: 2,
            stress_cost: 10,
            ideal_image_tags: vec![(ImageTag::Pure, 40)],
            conflicting_image_tags: vec![],
            personality_preference: None,
            skill_gains: vec![],
        };

        let skills = ProfessionalSkills {
            vocal: 3000,
            dance: 2000,
            poise: 1000,
            ..Default::default()
        };
        let image = ImageTags {
            pure: 50,
            ..Default::default()
        };
        let personality = PersonalitySpectrums::default();

        // weighted_sum = 3000*0.6 + 2000*0.3 + 1000*0.1 = 1800+600+100 = 2500
        // pure=50 >= 40 → image_mod=1.05
        // no preference → personality_mod=1.0
        // popularity=50 → popularity_mod=1.0
        // score = (2500*1.05*1.0*1.0) as i32 = 2625
        let score = gig.calculate_success_score(&skills, &image, &personality, 50);
        assert_eq!(score, 2625);
    }

    #[test]
    fn gig_pay_scales_with_popularity() {
        let gig = GigDef {
            id: GigId(2),
            name: "Ad Campaign".to_string(),
            category: GigCategory::Endorsement,
            duration_weeks: 1,
            required_recognition_tier: RecognitionTier::Unknown,
            skill_weights: vec![],
            base_pay: 50_000,
            recognition_reward: 0,
            reputation_reward: 0,
            stress_cost: 5,
            ideal_image_tags: vec![],
            conflicting_image_tags: vec![],
            personality_preference: None,
            skill_gains: vec![],
        };

        // pop=0: 1.0 + (0-50)/200 = 1.0 - 0.25 = 0.75 → 50000*0.75 = 37500
        assert_eq!(gig.calculate_pay(0), 37_500);
        // pop=50: 1.0 + 0/200 = 1.0 → 50000
        assert_eq!(gig.calculate_pay(50), 50_000);
        // pop=100: 1.0 + 50/200 = 1.25 → 62500
        assert_eq!(gig.calculate_pay(100), 62_500);
    }

    #[test]
    fn serialization_roundtrip() {
        let gig = make_music_gig();
        let serialized = ron::to_string(&gig).unwrap();
        let deserialized: GigDef = ron::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, gig.id);
        assert_eq!(deserialized.name, gig.name);
        assert_eq!(deserialized.category, gig.category);
        assert_eq!(deserialized.duration_weeks, gig.duration_weeks);
        assert_eq!(deserialized.base_pay, gig.base_pay);
        assert_eq!(deserialized.skill_weights.len(), gig.skill_weights.len());
    }
}
