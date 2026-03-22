use crate::artist::Artist;
use crate::gig::GigDef;
use crate::job::JobDef;
use crate::training::TrainingDef;
use crate::types::Activity;

const REST_STRESS_REDUCTION: i32 = 20;

pub fn apply_training(artist: &mut Artist, training: &TrainingDef) -> i64 {
    let current_skill = artist.skills.get(training.skill);
    let tier_idx = training.best_tier_index(current_skill);
    let effect = training.calculate_effect(tier_idx, &artist.base_attributes, artist.stats.stress);
    artist.skills.apply_gain(training.skill, effect.skill_gain);
    artist.stats.stress = (artist.stats.stress + effect.stress_increase).min(100);
    artist.current_activity = Activity::Training;
    effect.cost
}

pub fn apply_job(artist: &mut Artist, job: &JobDef) -> i64 {
    let effect = job.calculate_effect();
    for (target, gain) in &effect.skill_gains {
        artist.skills.apply_gain(*target, *gain);
    }
    for (target, loss) in &effect.skill_losses {
        artist.skills.apply_loss(*target, *loss);
    }
    artist.stats.add_recognition(effect.recognition_gain);
    artist.stats.stress = (artist.stats.stress + effect.stress_change).clamp(0, 100);
    artist.current_activity = Activity::PartTimeJob;
    effect.pay
}

pub fn apply_rest(artist: &mut Artist) {
    artist.stats.stress = (artist.stats.stress - REST_STRESS_REDUCTION).max(0);
    artist.current_activity = Activity::Rest;
}

pub fn start_gig(artist: &mut Artist, gig: &GigDef) {
    artist.locked_weeks = gig.duration_weeks;
    artist.current_activity = Activity::Gig;
}

pub fn complete_gig(artist: &mut Artist, gig: &GigDef) -> i64 {
    for (target, gain) in &gig.skill_gains {
        artist.skills.apply_gain(*target, *gain);
    }
    artist.stats.add_recognition(gig.recognition_reward);
    artist.stats.reputation = (artist.stats.reputation + gig.reputation_reward).clamp(-100, 100);
    artist.stats.stress = (artist.stats.stress + gig.stress_cost).min(100);
    gig.calculate_pay(artist.stats.popularity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artist::Artist;
    use crate::attribute::BaseAttributes;
    use crate::gig::{GigCategory, GigDef};
    use crate::job::JobDef;
    use crate::stats::RecognitionTier;
    use crate::training::{PrimaryAttribute, SkillTarget, TrainingDef, TrainingTier};
    use crate::types::{Activity, ArtistId, GigId, JobId, TrainingId};

    fn make_artist() -> Artist {
        Artist::new(
            ArtistId(1),
            "Test Artist".to_string(),
            20,
            BaseAttributes::default(), // all 50
        )
    }

    fn make_vocal_training() -> TrainingDef {
        // attrs all 50 → primary_bonus=0, secondary_bonus=0 → attr_bonus=1.0
        // stress=0 → condition=1.0
        // gain = 40 * 1.0 * 1.0 = 40, stress_increase=5, cost=8000
        TrainingDef {
            id: TrainingId(1),
            name: "Vocal Training".to_string(),
            skill: SkillTarget::Vocal,
            tiers: vec![TrainingTier {
                cost: 8000,
                base_gain: 40,
                stress_increase: 5,
                unlock_threshold: 0,
            }],
            primary_attribute: PrimaryAttribute::Empathy,
            secondary_attribute: None,
        }
    }

    fn make_job() -> JobDef {
        JobDef {
            id: JobId(1),
            name: "Convenience Store".to_string(),
            pay: 600,
            skill_gains: vec![(SkillTarget::Vocal, 15)],
            skill_losses: vec![(SkillTarget::Poise, 5)],
            recognition_gain: 3,
            stress_change: 3,
            required_recognition_tier: RecognitionTier::Unknown,
        }
    }

    fn make_gig() -> GigDef {
        // base_pay=100000, pop=50 → pay_mod=1.0 → pay=100000
        GigDef {
            id: GigId(1),
            name: "Drama Lead".to_string(),
            category: GigCategory::FilmTv,
            duration_weeks: 4,
            required_recognition_tier: RecognitionTier::Newcomer,
            skill_weights: vec![],
            base_pay: 100_000,
            recognition_reward: 50,
            reputation_reward: 3,
            stress_cost: 15,
            ideal_image_tags: vec![],
            conflicting_image_tags: vec![],
            personality_preference: None,
            skill_gains: vec![(SkillTarget::Acting, 50)],
        }
    }

    #[test]
    fn apply_training_increases_skill_and_stress() {
        let mut artist = make_artist();
        // vocal starts at 0, stress starts at 0
        let cost = apply_training(&mut artist, &make_vocal_training());
        assert_eq!(artist.skills.vocal, 40);
        assert_eq!(artist.stats.stress, 5);
        assert_eq!(cost, 8000);
        assert_eq!(artist.current_activity, Activity::Training);
    }

    #[test]
    fn apply_job_increases_and_decreases_skills() {
        let mut artist = make_artist();
        artist.skills.poise = 10; // so poise can go down by 5
        let pay = apply_job(&mut artist, &make_job());
        assert_eq!(artist.skills.vocal, 15);
        assert_eq!(artist.skills.poise, 5);
        assert_eq!(artist.stats.recognition, 3);
        assert_eq!(artist.stats.stress, 3);
        assert_eq!(pay, 600);
        assert_eq!(artist.current_activity, Activity::PartTimeJob);
    }

    #[test]
    fn apply_rest_reduces_stress() {
        let mut artist = make_artist();
        artist.stats.stress = 40;
        apply_rest(&mut artist);
        assert_eq!(artist.stats.stress, 20);
        assert_eq!(artist.current_activity, Activity::Rest);
    }

    #[test]
    fn start_gig_locks_artist() {
        let mut artist = make_artist();
        start_gig(&mut artist, &make_gig());
        assert_eq!(artist.locked_weeks, 4);
        assert!(artist.is_locked());
        assert_eq!(artist.current_activity, Activity::Gig);
    }

    #[test]
    fn skill_gain_clamps_at_max() {
        // Pre-set vocal to just below SKILL_MAX, then apply training that would
        // push it over the ceiling and verify it is clamped to exactly SKILL_MAX.
        use crate::attribute::SKILL_MAX;
        let mut artist = make_artist();
        // Place the skill one point below the maximum so any positive gain overflows.
        artist.skills.vocal = SKILL_MAX - 1;
        // Ensure stress is 0 so the training produces a positive gain (gain=40).
        artist.stats.stress = 0;
        apply_training(&mut artist, &make_vocal_training());
        assert_eq!(
            artist.skills.vocal, SKILL_MAX,
            "vocal should be clamped at SKILL_MAX, not {}",
            artist.skills.vocal
        );
    }

    #[test]
    fn complete_gig_rewards() {
        let mut artist = make_artist();
        // popularity=50 → pay_mod=1.0 → pay=100000
        artist.stats.popularity = 50;
        let pay = complete_gig(&mut artist, &make_gig());
        assert_eq!(artist.skills.acting, 50);
        assert_eq!(artist.stats.recognition, 50);
        assert_eq!(artist.stats.reputation, 3);
        assert_eq!(artist.stats.stress, 15);
        assert_eq!(pay, 100_000);
    }
}
