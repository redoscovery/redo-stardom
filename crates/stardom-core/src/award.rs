use serde::{Deserialize, Serialize};

use crate::attribute::ProfessionalSkills;
use crate::gig::GigCategory;
use crate::persona::{ImageTag, ImageTags};
use crate::training::SkillTarget;
use crate::types::AwardId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwardDef {
    pub id: AwardId,
    pub name: String,
    pub ceremony_month: u32,
    pub nomination_month: u32,
    pub scoring_skills: Vec<(SkillTarget, f64)>,
    pub scoring_image_tags: Vec<(ImageTag, f64)>,
    pub requires_gig_category: Option<GigCategory>,
    pub ai_competitor_score: i32,
    pub recognition_boost: i64,
    pub reputation_boost: i32,
}

impl AwardDef {
    /// Skills contribute directly, image tags are scaled ×100 to match skill range.
    pub fn calculate_score(&self, skills: &ProfessionalSkills, image: &ImageTags) -> i32 {
        let skill_score: f64 = self
            .scoring_skills
            .iter()
            .map(|(t, w)| skills.get(*t) as f64 * w)
            .sum();
        let image_score: f64 = self
            .scoring_image_tags
            .iter()
            .map(|(t, w)| t.value_from(image) as f64 * w * 100.0)
            .sum();
        (skill_score + image_score) as i32
    }

    pub fn is_nominated(&self, completed_categories: &[GigCategory]) -> bool {
        match &self.requires_gig_category {
            None => true,
            Some(cat) => completed_categories.contains(cat),
        }
    }

    pub fn is_winner(&self, artist_score: i32) -> bool {
        artist_score > self.ai_competitor_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::ProfessionalSkills;
    use crate::gig::GigCategory;
    use crate::persona::{ImageTag, ImageTags};
    use crate::training::SkillTarget;
    use crate::types::AwardId;

    fn make_award(
        requires_gig_category: Option<GigCategory>,
        ai_competitor_score: i32,
    ) -> AwardDef {
        AwardDef {
            id: AwardId(1),
            name: "Best Model".to_string(),
            ceremony_month: 9,
            nomination_month: 8,
            scoring_skills: vec![(SkillTarget::Poise, 1.0)],
            scoring_image_tags: vec![
                (ImageTag::Sexy, 0.5),
                (ImageTag::Cool, 0.3),
            ],
            requires_gig_category,
            ai_competitor_score,
            recognition_boost: 500,
            reputation_boost: 5,
        }
    }

    #[test]
    fn award_score_calculation() {
        // poise=4000 weight 1.0 → 4000
        // sexy=60 weight 0.5 × 100 → 3000
        // cool=40 weight 0.3 × 100 → 1200
        // total = 4000 + 3000 + 1200 = 8200
        let award = make_award(Some(GigCategory::Modeling), 3000);
        let skills = ProfessionalSkills {
            poise: 4000,
            ..Default::default()
        };
        let image = ImageTags {
            sexy: 60,
            cool: 40,
            ..Default::default()
        };
        let score = award.calculate_score(&skills, &image);
        assert_eq!(score, 8200);
    }

    #[test]
    fn award_nomination_requires_matching_gig() {
        // Award requires Modeling; artist has only completed Music
        let award = make_award(Some(GigCategory::Modeling), 3000);
        assert!(!award.is_nominated(&[GigCategory::Music]));
        assert!(award.is_nominated(&[GigCategory::Music, GigCategory::Modeling]));
    }

    #[test]
    fn award_wins_when_beating_ai() {
        let award = make_award(None, 3000);
        assert!(award.is_winner(3500));
        assert!(!award.is_winner(2500));
    }

    #[test]
    fn award_no_category_requirement() {
        let award = make_award(None, 3000);
        // No category requirement → always nominated regardless of history
        assert!(award.is_nominated(&[]));
        assert!(award.is_nominated(&[GigCategory::Music]));
    }

    #[test]
    fn serialization_roundtrip() {
        let award = make_award(Some(GigCategory::Modeling), 3000);
        let serialized = ron::to_string(&award).unwrap();
        let deserialized: AwardDef = ron::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, award.id);
        assert_eq!(deserialized.name, award.name);
        assert_eq!(deserialized.ceremony_month, award.ceremony_month);
        assert_eq!(deserialized.nomination_month, award.nomination_month);
        assert_eq!(deserialized.ai_competitor_score, award.ai_competitor_score);
        assert_eq!(deserialized.recognition_boost, award.recognition_boost);
        assert_eq!(deserialized.reputation_boost, award.reputation_boost);
    }
}
