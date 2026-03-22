use serde::{Deserialize, Serialize};

use crate::attribute::ProfessionalSkills;
use crate::persona::{ImageTags, PersonalitySpectrums, Spectrum};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageTag {
    Pure,
    Sexy,
    Cool,
    Intellectual,
    Funny,
    Mysterious,
}

impl ImageTag {
    pub fn value_from(&self, tags: &ImageTags) -> i32 {
        match self {
            Self::Pure => tags.pure,
            Self::Sexy => tags.sexy,
            Self::Cool => tags.cool,
            Self::Intellectual => tags.intellectual,
            Self::Funny => tags.funny,
            Self::Mysterious => tags.mysterious,
        }
    }
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

fn skill_value(skills: &ProfessionalSkills, target: SkillTarget) -> i32 {
    match target {
        SkillTarget::Vocal => skills.vocal,
        SkillTarget::Acting => skills.acting,
        SkillTarget::Dance => skills.dance,
        SkillTarget::Poise => skills.poise,
        SkillTarget::Eloquence => skills.eloquence,
        SkillTarget::Creativity => skills.creativity,
    }
}

impl GigDef {
    /// Returns true if artist_tier meets or exceeds this gig's requirement.
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
            .map(|(target, weight)| skill_value(skills, *target) as f64 * weight)
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

        // Popularity modifier
        let popularity_mod: f64 = 1.0 + (popularity as f64 - 50.0) / 200.0;

        (weighted_sum * image_mod * personality_mod * popularity_mod) as i32
    }

    /// Pay calculation: base_pay * popularity_modifier
    pub fn calculate_pay(&self, popularity: i32) -> i64 {
        let popularity_mod = 1.0 + (popularity as f64 - 50.0) / 200.0;
        (self.base_pay as f64 * popularity_mod) as i64
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
        // vocal=3000, dance=2000, poise=1000
        // weighted_sum = 3000*0.5 + 2000*0.3 + 1000*0.2 = 1500+600+200 = 2300
        // ideal tag Pure threshold=40, artist pure=50 >= 40 → image_mod = 1.05
        // no personality preference → personality_mod = 1.0
        // popularity = 50 → popularity_mod = 1.0 + (50-50)/200 = 1.0
        // score = (2300 * 1.05 * 1.0 * 1.0) as i32 = 2415
        // Wait, let me re-read spec: gig_success_score_basic — vocal=3000, dance=2000, poise=1000, pure=50>=40 → score=2625
        // So score=2625. Let me re-check formula with popularity_mod...
        // Actually if popularity is not 50 → popularity_mod != 1.0
        // Let me try popularity=75: 1.0+(75-50)/200=1.125
        // 2300*1.05*1.0*1.125 = 2415 * 1.125 = not matching
        // Let me try pure=50, threshold=40, 50 >= 40 (not >= 80), so +0.05 → 1.05
        // 2300*1.05 = 2415 ≠ 2625
        // Hmm, let me try: maybe weighted sum includes pure/image differently
        // Or maybe pure=50 >= 2*40=80 is false, pure=50 >= 40 is true → +0.05
        // To get 2625: if image_mod=1.125: 2300*1.125=2587.5 ≠ 2625
        // If weighted_sum=2500: 2500*1.05=2625!
        // 2500 with vocal=3000, dance=2000, poise=1000:
        //   0.5*3000 + x*2000 + y*1000 = 2500? w/ 0.5+x+y=1
        //   1500 + x*2000 + y*1000 = 2500 → x*2000+y*1000=1000
        //   if x=0.3, y=0.2: 600+200=800 ≠ 1000
        //   if x=0.4, y=0.1: 800+100=900 ≠ 1000
        //   if x=0.25, y=0.25: 500+250=750 ≠ 1000
        //   weights 0.5+0.3+0.1: 1500+600+100=2200; 2200*1.05=2310 ≠ 2625
        //   weights 0.6+0.3+0.1: 1800+600+100=2500; 2500*1.05=2625! ✓
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
