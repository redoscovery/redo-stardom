use crate::gig::{GigCategory, GigDef};
use crate::stats::RecognitionTier;

fn is_rotation_a_category(cat: GigCategory) -> bool {
    matches!(cat, GigCategory::Music | GigCategory::FilmTv)
}

pub fn generate_pool(catalog: &[GigDef], rotation_a: bool) -> Vec<&GigDef> {
    catalog
        .iter()
        .filter(|g| is_rotation_a_category(g.category) == rotation_a)
        .collect()
}

pub fn filter_available<'a>(pool: &[&'a GigDef], artist_tier: RecognitionTier) -> Vec<&'a GigDef> {
    pool.iter()
        .filter(|g| g.is_available(artist_tier))
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gig::{GigCategory, GigDef};
    use crate::stats::RecognitionTier;
    use crate::training::SkillTarget;
    use crate::types::GigId;

    fn make_catalog() -> Vec<GigDef> {
        vec![
            GigDef {
                id: GigId(1),
                name: "Indie Concert".to_string(),
                category: GigCategory::Music,
                duration_weeks: 1,
                required_recognition_tier: RecognitionTier::Unknown,
                skill_weights: vec![(SkillTarget::Vocal, 1.0)],
                base_pay: 10_000,
                recognition_reward: 10,
                reputation_reward: 1,
                stress_cost: 5,
                ideal_image_tags: vec![],
                conflicting_image_tags: vec![],
                personality_preference: None,
                skill_gains: vec![],
            },
            GigDef {
                id: GigId(2),
                name: "TV Drama".to_string(),
                category: GigCategory::FilmTv,
                duration_weeks: 4,
                required_recognition_tier: RecognitionTier::Rising,
                skill_weights: vec![(SkillTarget::Acting, 1.0)],
                base_pay: 50_000,
                recognition_reward: 30,
                reputation_reward: 3,
                stress_cost: 10,
                ideal_image_tags: vec![],
                conflicting_image_tags: vec![],
                personality_preference: None,
                skill_gains: vec![],
            },
            GigDef {
                id: GigId(3),
                name: "Brand Endorsement".to_string(),
                category: GigCategory::Endorsement,
                duration_weeks: 1,
                required_recognition_tier: RecognitionTier::Unknown,
                skill_weights: vec![(SkillTarget::Poise, 1.0)],
                base_pay: 20_000,
                recognition_reward: 5,
                reputation_reward: 1,
                stress_cost: 3,
                ideal_image_tags: vec![],
                conflicting_image_tags: vec![],
                personality_preference: None,
                skill_gains: vec![],
            },
        ]
    }

    #[test]
    fn generate_pool_filters_by_rotation() {
        let catalog = make_catalog();

        let pool_a = generate_pool(&catalog, true);
        assert_eq!(pool_a.len(), 2);
        assert!(pool_a
            .iter()
            .all(|g| matches!(g.category, GigCategory::Music | GigCategory::FilmTv)));

        let pool_b = generate_pool(&catalog, false);
        assert_eq!(pool_b.len(), 1);
        assert!(pool_b
            .iter()
            .all(|g| !matches!(g.category, GigCategory::Music | GigCategory::FilmTv)));
    }

    #[test]
    fn filter_available_for_artist() {
        let catalog = make_catalog();
        // Rotation A contains Music (Unknown tier) and FilmTv (Rising tier)
        let pool_a = generate_pool(&catalog, true);

        // Unknown tier artist can only see Unknown-tier gigs
        let available = filter_available(&pool_a, RecognitionTier::Unknown);
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].id, GigId(1));
    }
}
