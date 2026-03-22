use serde::{Deserialize, Serialize};

use crate::stats::RecognitionTier;
use crate::training::SkillTarget;
use crate::types::JobId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDef {
    pub id: JobId,
    pub name: String,
    pub pay: i64,
    pub skill_gains: Vec<(SkillTarget, i32)>,
    pub skill_losses: Vec<(SkillTarget, i32)>,
    pub recognition_gain: i64,
    pub stress_change: i32,
    pub required_recognition_tier: RecognitionTier,
}

pub struct JobEffect {
    pub pay: i64,
    pub skill_gains: Vec<(SkillTarget, i32)>,
    pub skill_losses: Vec<(SkillTarget, i32)>,
    pub recognition_gain: i64,
    pub stress_change: i32,
}

impl JobDef {
    pub fn is_available(&self, artist_tier: RecognitionTier) -> bool {
        artist_tier >= self.required_recognition_tier
    }

    pub fn calculate_effect(&self) -> JobEffect {
        JobEffect {
            pay: self.pay,
            skill_gains: self.skill_gains.clone(),
            skill_losses: self.skill_losses.clone(),
            recognition_gain: self.recognition_gain,
            stress_change: self.stress_change,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_unknown_job() -> JobDef {
        JobDef {
            id: JobId(1),
            name: "Convenience Store Clerk".to_string(),
            pay: 5000,
            skill_gains: vec![(SkillTarget::Eloquence, 5)],
            skill_losses: vec![],
            recognition_gain: 2,
            stress_change: 3,
            required_recognition_tier: RecognitionTier::Unknown,
        }
    }

    fn make_rising_job() -> JobDef {
        JobDef {
            id: JobId(2),
            name: "TV Commercial".to_string(),
            pay: 20000,
            skill_gains: vec![(SkillTarget::Poise, 10), (SkillTarget::Eloquence, 5)],
            skill_losses: vec![],
            recognition_gain: 15,
            stress_change: 5,
            required_recognition_tier: RecognitionTier::Rising,
        }
    }

    #[test]
    fn job_is_available_at_tier() {
        let job = make_unknown_job();
        assert!(job.is_available(RecognitionTier::Unknown));
        assert!(job.is_available(RecognitionTier::Rising));
    }

    #[test]
    fn job_not_available_below_tier() {
        let job = make_rising_job();
        assert!(!job.is_available(RecognitionTier::Unknown));
        assert!(!job.is_available(RecognitionTier::Newcomer));
        assert!(job.is_available(RecognitionTier::Rising));
        assert!(job.is_available(RecognitionTier::Established));
    }

    #[test]
    fn job_effect() {
        let job = make_rising_job();
        let effect = job.calculate_effect();
        assert_eq!(effect.pay, 20000);
        assert_eq!(effect.recognition_gain, 15);
        assert_eq!(effect.stress_change, 5);
        assert_eq!(effect.skill_gains.len(), 2);
        assert_eq!(effect.skill_gains[0], (SkillTarget::Poise, 10));
        assert_eq!(effect.skill_gains[1], (SkillTarget::Eloquence, 5));
        assert!(effect.skill_losses.is_empty());
    }

    #[test]
    fn serialization_roundtrip() {
        let job = make_rising_job();
        let serialized = ron::to_string(&job).unwrap();
        let deserialized: JobDef = ron::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, job.id);
        assert_eq!(deserialized.name, job.name);
        assert_eq!(deserialized.pay, job.pay);
        assert_eq!(
            deserialized.required_recognition_tier,
            job.required_recognition_tier
        );
        assert_eq!(deserialized.skill_gains.len(), job.skill_gains.len());
    }
}
