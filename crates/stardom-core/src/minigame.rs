use serde::{Deserialize, Serialize};

use crate::training::SkillTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MiniGameId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MiniGameCategory {
    Rhythm,
    Reaction,
    Memory,
    Trivia,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiniGameRating {
    Failed,
    Standard,
    Excellent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniGameDef {
    pub id: MiniGameId,
    pub name: String,
    pub category: MiniGameCategory,
    pub difficulty_threshold: i32,
    pub relevant_skill: SkillTarget,
}

pub struct MiniGameResult {
    pub passed: bool,
    pub rating: MiniGameRating,
    pub score: i32,
}

/// Spec 5.4: score = skill * 0.7 + random(0, skill * 0.3). roll is 0.0..1.0.
pub fn auto_resolve(skill: i32, threshold: i32, roll: f64) -> MiniGameResult {
    let score = (skill as f64 * 0.7 + skill as f64 * 0.3 * roll.clamp(0.0, 1.0)) as i32;
    let passed = score >= threshold;
    let rating = if !passed {
        MiniGameRating::Failed
    } else if score >= (threshold as f64 * 1.2) as i32 {
        MiniGameRating::Excellent
    } else {
        MiniGameRating::Standard
    };
    MiniGameResult {
        passed,
        rating,
        score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_resolve_passes_with_high_skill() {
        // skill=5000, threshold=3000, roll=0
        // score = 5000*0.7 + 5000*0.3*0 = 3500
        // passed: 3500 >= 3000 → true
        // excellent threshold: 3000 * 1.2 = 3600 → 3500 < 3600 → Standard
        let result = auto_resolve(5000, 3000, 0.0);
        assert!(result.passed);
        assert_eq!(result.rating, MiniGameRating::Standard);
        assert_eq!(result.score, 3500);
    }

    #[test]
    fn auto_resolve_excellent_at_120pct() {
        // skill=5000, threshold=3000, roll=1.0
        // score = 5000*0.7 + 5000*0.3*1.0 = 3500 + 1500 = 5000
        // excellent threshold: 3000 * 1.2 = 3600 → 5000 >= 3600 → Excellent
        let result = auto_resolve(5000, 3000, 1.0);
        assert!(result.passed);
        assert_eq!(result.rating, MiniGameRating::Excellent);
        assert_eq!(result.score, 5000);
    }

    #[test]
    fn auto_resolve_fails_with_low_skill() {
        // skill=1000, threshold=3000, roll=1.0
        // score = 1000*0.7 + 1000*0.3*1.0 = 700 + 300 = 1000
        // passed: 1000 >= 3000 → false → Failed
        let result = auto_resolve(1000, 3000, 1.0);
        assert!(!result.passed);
        assert_eq!(result.rating, MiniGameRating::Failed);
        assert_eq!(result.score, 1000);
    }

    #[test]
    fn minigame_def_serialization() {
        let def = MiniGameDef {
            id: MiniGameId(1),
            name: "Beat Sync".to_string(),
            category: MiniGameCategory::Rhythm,
            difficulty_threshold: 2500,
            relevant_skill: SkillTarget::Dance,
        };
        let serialized = ron::to_string(&def).unwrap();
        let deserialized: MiniGameDef = ron::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, def.id);
        assert_eq!(deserialized.name, def.name);
        assert_eq!(deserialized.category, def.category);
        assert_eq!(deserialized.difficulty_threshold, def.difficulty_threshold);
        assert_eq!(deserialized.relevant_skill, def.relevant_skill);
    }
}
