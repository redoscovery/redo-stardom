# Phase 2: Activities — Training, Jobs, Gigs & Scheduling

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the three activity types (training, part-time jobs, gigs), wire up the `AssignActivity` command so the player can schedule weekly activities for artists, connect gig income/training costs to the financial system, and update the game loop to process activities and resolve gigs.

**Architecture:** All activity definitions are data-driven (RON files). New modules `training.rs`, `job.rs`, `gig.rs`, and `scheduling.rs` in `stardom-core`. The game loop in `game.rs` is extended to process assigned activities each week. The `Activity` enum from `types.rs` gains associated data via new wrapper types.

**Tech Stack:** Rust 1.94, serde + ron, cargo test

**Spec reference:** `docs/superpowers/specs/2026-03-22-redo-stardom-design.md` (Sections 4.1–4.4, 4.6, Appendix A.1–A.5, A.10)

---

## File Structure

```
crates/stardom-core/src/
├── training.rs      # TrainingDef, TrainingTier, training effect calculation
├── job.rs           # JobDef, job effect calculation
├── gig.rs           # GigDef, GigCategory, gig success/reward calculation, GigState
├── scheduling.rs    # schedule_activity(), resolve weekly activities
├── game.rs          # (modify) add AssignActivity command, integrate scheduling
├── artist.rs        # (modify) add locked_weeks field for multi-week gigs
├── types.rs         # (modify) add TrainingId, JobId
├── lib.rs           # (modify) add new pub mod declarations
data/
├── training/
│   └── default_training.ron   # sample training definitions
├── jobs/
│   └── default_jobs.ron       # sample job definitions
└── gigs/
    └── default_gigs.ron       # sample gig definitions
```

---

## Task 1: Add New ID Types & Extend Artist

**Files:**
- Modify: `crates/stardom-core/src/types.rs`
- Modify: `crates/stardom-core/src/artist.rs`

- [ ] **Step 1: Write failing tests for new IDs and artist locked_weeks**

In `types.rs`, add to existing test module:
```rust
#[test]
fn training_id_serialization() {
    let id = TrainingId(5);
    let s = ron::to_string(&id).unwrap();
    let d: TrainingId = ron::from_str(&s).unwrap();
    assert_eq!(id, d);
}
```

In `artist.rs`, add test:
```rust
#[test]
fn artist_locked_in_gig() {
    let mut artist = make_artist();
    assert!(!artist.is_locked());
    artist.locked_weeks = 3;
    assert!(artist.is_locked());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p stardom-core types::tests::training_id artist::tests::artist_locked`
Expected: FAIL

- [ ] **Step 3: Implement**

In `types.rs`, extend the `id_newtype!` macro call:
```rust
id_newtype!(ArtistId, GigId, OutfitId, CrisisId, TrainingId, JobId);
```

In `artist.rs`, add field and method:
```rust
// Add to Artist struct:
pub locked_weeks: u32, // remaining weeks locked in a multi-week gig (0 = free)

// Add to Artist::new():
locked_weeks: 0,

// Add method:
pub fn is_locked(&self) -> bool {
    self.locked_weeks > 0
}
```

Update `data_loader.rs` `into_artist` — no change needed since it delegates to `Artist::new`.

- [ ] **Step 4: Run tests**

Run: `cargo test -p stardom-core`
Expected: all tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/stardom-core/src/types.rs crates/stardom-core/src/artist.rs
git commit -m "feat(core): add TrainingId, JobId and artist locked_weeks"
```

---

## Task 2: Training Definitions & Effect Calculation

**Files:**
- Create: `crates/stardom-core/src/training.rs`
- Modify: `crates/stardom-core/src/lib.rs` (add `pub mod training;`)
- Create: `data/training/default_training.ron`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::BaseAttributes;
    use crate::stats::AuxiliaryStats;

    fn sample_training() -> TrainingDef {
        TrainingDef {
            id: TrainingId(1),
            name: "Vocal Lesson".to_string(),
            skill: SkillTarget::Vocal,
            tiers: vec![
                TrainingTier { cost: 8_000, base_gain: 40, stress_increase: 5, unlock_threshold: 0 },
                TrainingTier { cost: 16_000, base_gain: 80, stress_increase: 10, unlock_threshold: 1_000 },
                TrainingTier { cost: 28_000, base_gain: 130, stress_increase: 16, unlock_threshold: 3_000 },
                TrainingTier { cost: 44_000, base_gain: 180, stress_increase: 22, unlock_threshold: 6_000 },
            ],
            primary_attribute: PrimaryAttribute::Empathy,
            secondary_attribute: Some(PrimaryAttribute::Charm),
        }
    }

    #[test]
    fn best_available_tier() {
        let t = sample_training();
        assert_eq!(t.best_tier_index(0), 0);
        assert_eq!(t.best_tier_index(999), 0);
        assert_eq!(t.best_tier_index(1_000), 1);
        assert_eq!(t.best_tier_index(6_500), 3);
    }

    #[test]
    fn training_effect_basic() {
        let t = sample_training();
        let attrs = BaseAttributes::new(50, 50, 50, 50);
        let stress = 0;
        let result = t.calculate_effect(0, &attrs, stress);
        // tier 0: base_gain=40, attr_bonus=(50-50)/100=0.0, condition=1.0
        // effective_gain = 40 * 1.0 * 1.0 = 40
        assert_eq!(result.skill_gain, 40);
        assert_eq!(result.stress_increase, 5);
        assert_eq!(result.cost, 8_000);
    }

    #[test]
    fn training_effect_with_high_attribute() {
        let t = sample_training();
        let attrs = BaseAttributes::new(50, 50, 80, 70); // empathy=80 (primary), charm=70 (secondary)
        let stress = 0;
        let result = t.calculate_effect(0, &attrs, stress);
        // primary_bonus = (80-50)/100 = 0.30
        // secondary_bonus = (70-50)/200 = 0.10
        // total = 0.40
        // effective_gain = 40 * 1.40 * 1.0 = 56
        assert_eq!(result.skill_gain, 56);
    }

    #[test]
    fn training_effect_under_stress() {
        let t = sample_training();
        let attrs = BaseAttributes::new(50, 50, 50, 50);
        let stress = 45; // condition_modifier = 0.85
        let result = t.calculate_effect(0, &attrs, stress);
        // 40 * 1.0 * 0.85 = 34
        assert_eq!(result.skill_gain, 34);
    }

    #[test]
    fn serialization_roundtrip() {
        let t = sample_training();
        let s = ron::to_string(&t).unwrap();
        let d: TrainingDef = ron::from_str(&s).unwrap();
        assert_eq!(d.name, "Vocal Lesson");
        assert_eq!(d.tiers.len(), 4);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p stardom-core training`
Expected: FAIL

- [ ] **Step 3: Implement training module**

```rust
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
    /// Returns the index of the best available tier for the given skill level.
    pub fn best_tier_index(&self, current_skill: i32) -> usize {
        let mut best = 0;
        for (i, tier) in self.tiers.iter().enumerate() {
            if current_skill >= tier.unlock_threshold {
                best = i;
            }
        }
        best
    }

    /// Calculate training effect using spec A.1 formula.
    pub fn calculate_effect(
        &self,
        tier_index: usize,
        base_attrs: &BaseAttributes,
        stress: i32,
    ) -> TrainingEffect {
        let tier = &self.tiers[tier_index];

        let primary_bonus =
            (self.primary_attribute.value_from(base_attrs) - 50) as f64 / 100.0;
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
```

- [ ] **Step 4: Add `pub mod training;` to lib.rs**

- [ ] **Step 5: Create sample data file `data/training/default_training.ron`**

```ron
[
    TrainingDef(
        id: TrainingId(1),
        name: "Vocal Lesson",
        skill: Vocal,
        tiers: [
            TrainingTier(cost: 8000, base_gain: 40, stress_increase: 5, unlock_threshold: 0),
            TrainingTier(cost: 16000, base_gain: 80, stress_increase: 10, unlock_threshold: 1000),
            TrainingTier(cost: 28000, base_gain: 130, stress_increase: 16, unlock_threshold: 3000),
            TrainingTier(cost: 44000, base_gain: 180, stress_increase: 22, unlock_threshold: 6000),
        ],
        primary_attribute: Empathy,
        secondary_attribute: Some(Charm),
    ),
    TrainingDef(
        id: TrainingId(2),
        name: "Acting Workshop",
        skill: Acting,
        tiers: [
            TrainingTier(cost: 8000, base_gain: 40, stress_increase: 5, unlock_threshold: 0),
            TrainingTier(cost: 16000, base_gain: 80, stress_increase: 10, unlock_threshold: 1000),
            TrainingTier(cost: 28000, base_gain: 130, stress_increase: 16, unlock_threshold: 3000),
            TrainingTier(cost: 44000, base_gain: 180, stress_increase: 22, unlock_threshold: 6000),
        ],
        primary_attribute: Empathy,
        secondary_attribute: Some(Stamina),
    ),
    TrainingDef(
        id: TrainingId(3),
        name: "Dance Class",
        skill: Dance,
        tiers: [
            TrainingTier(cost: 8000, base_gain: 40, stress_increase: 5, unlock_threshold: 0),
            TrainingTier(cost: 16000, base_gain: 80, stress_increase: 10, unlock_threshold: 1000),
            TrainingTier(cost: 28000, base_gain: 130, stress_increase: 16, unlock_threshold: 3000),
            TrainingTier(cost: 44000, base_gain: 180, stress_increase: 22, unlock_threshold: 6000),
        ],
        primary_attribute: Stamina,
        secondary_attribute: Some(Charm),
    ),
]
```

- [ ] **Step 6: Run tests**

Run: `cargo test -p stardom-core training`
Expected: 5 tests PASS

- [ ] **Step 7: Commit**

```bash
git add crates/stardom-core/src/training.rs crates/stardom-core/src/lib.rs data/training/
git commit -m "feat(core): add training system with tier selection and efficiency formula"
```

---

## Task 3: Part-Time Job Definitions & Effect Calculation

**Files:**
- Create: `crates/stardom-core/src/job.rs`
- Modify: `crates/stardom-core/src/lib.rs` (add `pub mod job;`)
- Create: `data/jobs/default_jobs.ron`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_job() -> JobDef {
        JobDef {
            id: JobId(1),
            name: "Street Performance".to_string(),
            pay: 600,
            skill_gains: vec![(SkillTarget::Vocal, 15)],
            skill_losses: vec![(SkillTarget::Poise, 5)],
            recognition_gain: 3,
            stress_change: 3,
            required_recognition_tier: RecognitionTier::Unknown,
        }
    }

    #[test]
    fn job_is_available_at_tier() {
        let job = sample_job();
        assert!(job.is_available(RecognitionTier::Unknown));
        assert!(job.is_available(RecognitionTier::Rising));
    }

    #[test]
    fn job_not_available_below_tier() {
        let job = JobDef {
            required_recognition_tier: RecognitionTier::Rising,
            ..sample_job()
        };
        assert!(!job.is_available(RecognitionTier::Unknown));
        assert!(!job.is_available(RecognitionTier::Newcomer));
        assert!(job.is_available(RecognitionTier::Rising));
    }

    #[test]
    fn job_effect() {
        let job = sample_job();
        let effect = job.calculate_effect();
        assert_eq!(effect.pay, 600);
        assert_eq!(effect.skill_gains, vec![(SkillTarget::Vocal, 15)]);
        assert_eq!(effect.skill_losses, vec![(SkillTarget::Poise, 5)]);
        assert_eq!(effect.recognition_gain, 3);
        assert_eq!(effect.stress_change, 3);
    }

    #[test]
    fn serialization_roundtrip() {
        let job = sample_job();
        let s = ron::to_string(&job).unwrap();
        let d: JobDef = ron::from_str(&s).unwrap();
        assert_eq!(d.name, "Street Performance");
        assert_eq!(d.pay, 600);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p stardom-core job`
Expected: FAIL

- [ ] **Step 3: Implement job module**

```rust
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
```

**Important:** `RecognitionTier` in `stats.rs` must have `PartialOrd, Ord` derives added for `>=` to work. The enum variants are already in ascending order so derived ordering is correct. Change the derive to:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RecognitionTier { ... }
```

- [ ] **Step 4: Add `pub mod job;` to lib.rs**

- [ ] **Step 5: Create `data/jobs/default_jobs.ron`**

```ron
[
    JobDef(
        id: JobId(1),
        name: "Street Performance",
        pay: 600,
        skill_gains: [(Vocal, 15), (Eloquence, 5)],
        skill_losses: [(Poise, 5)],
        recognition_gain: 3,
        stress_change: 3,
        required_recognition_tier: Unknown,
    ),
    JobDef(
        id: JobId(2),
        name: "Background Extra",
        pay: 800,
        skill_gains: [(Acting, 10), (Dance, 5)],
        skill_losses: [(Creativity, 5)],
        recognition_gain: 2,
        stress_change: 4,
        required_recognition_tier: Unknown,
    ),
    JobDef(
        id: JobId(3),
        name: "Stage Show",
        pay: 1500,
        skill_gains: [(Vocal, 20), (Dance, 15)],
        skill_losses: [(Poise, 10)],
        recognition_gain: 6,
        stress_change: 8,
        required_recognition_tier: Newcomer,
    ),
]
```

- [ ] **Step 6: Run tests**

Run: `cargo test -p stardom-core job`
Expected: 4 tests PASS

- [ ] **Step 7: Commit**

```bash
git add crates/stardom-core/src/job.rs crates/stardom-core/src/lib.rs crates/stardom-core/src/stats.rs data/jobs/
git commit -m "feat(core): add part-time job system with tier-gated availability"
```

---

## Task 4: Gig Definitions & Success Calculation

**Files:**
- Create: `crates/stardom-core/src/gig.rs`
- Modify: `crates/stardom-core/src/lib.rs` (add `pub mod gig;`)
- Create: `data/gigs/default_gigs.ron`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::{BaseAttributes, ProfessionalSkills};
    use crate::persona::{ImageTags, PersonalitySpectrums, Spectrum};
    use crate::stats::{AuxiliaryStats, RecognitionTier};

    fn sample_gig() -> GigDef {
        GigDef {
            id: GigId(1),
            name: "Debut Single".to_string(),
            category: GigCategory::Music,
            duration_weeks: 2,
            required_recognition_tier: RecognitionTier::Newcomer,
            skill_weights: vec![(SkillTarget::Vocal, 0.6), (SkillTarget::Dance, 0.3), (SkillTarget::Poise, 0.1)],
            base_pay: 50_000,
            recognition_reward: 100,
            reputation_reward: 5,
            stress_cost: 10,
            ideal_image_tags: vec![(ImageTag::Pure, 40)],
            conflicting_image_tags: vec![],
            personality_preference: None,
            skill_gains: vec![(SkillTarget::Vocal, 30)],
        }
    }

    #[test]
    fn gig_availability() {
        let gig = sample_gig();
        assert!(!gig.is_available(RecognitionTier::Unknown));
        assert!(gig.is_available(RecognitionTier::Newcomer));
        assert!(gig.is_available(RecognitionTier::Star));
    }

    #[test]
    fn gig_success_score_basic() {
        let gig = sample_gig();
        let mut skills = ProfessionalSkills::default();
        skills.vocal = 3000;
        skills.dance = 2000;
        skills.poise = 1000;
        let image = ImageTags { pure: 50, ..Default::default() };
        let personality = PersonalitySpectrums::default();
        let popularity = 50;

        let score = gig.calculate_success_score(&skills, &image, &personality, popularity);
        // weighted_skill_sum = 3000*0.6 + 2000*0.3 + 1000*0.1 = 1800+600+100 = 2500
        // image: pure=50 >= 40 → +0.05. Total image_mod = 1.05, clamped to 0.80-1.20
        // personality: no preference → 1.0
        // popularity: 1.0 + (50-50)/200 = 1.0
        // score = 2500 * 1.05 * 1.0 * 1.0 = 2625
        assert_eq!(score, 2625);
    }

    #[test]
    fn gig_pay_scales_with_popularity() {
        let gig = sample_gig();
        let low_pop = gig.calculate_pay(0);   // 1.0 + (0-50)/200 = 0.75
        let mid_pop = gig.calculate_pay(50);  // 1.0
        let high_pop = gig.calculate_pay(100); // 1.25
        assert_eq!(low_pop, 37_500);  // 50000 * 0.75
        assert_eq!(mid_pop, 50_000);
        assert_eq!(high_pop, 62_500);
    }

    #[test]
    fn serialization_roundtrip() {
        let gig = sample_gig();
        let s = ron::to_string(&gig).unwrap();
        let d: GigDef = ron::from_str(&s).unwrap();
        assert_eq!(d.name, "Debut Single");
        assert_eq!(d.duration_weeks, 2);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p stardom-core gig`
Expected: FAIL

- [ ] **Step 3: Implement gig module**

```rust
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
    pub ideal_image_tags: Vec<(ImageTag, i32)>,        // (tag, threshold)
    pub conflicting_image_tags: Vec<(ImageTag, i32)>,   // (tag, threshold) — penalty if above
    pub personality_preference: Option<(Spectrum, i32)>, // (spectrum, preferred_min_value)
    pub skill_gains: Vec<(SkillTarget, i32)>,
}

impl GigDef {
    pub fn is_available(&self, artist_tier: RecognitionTier) -> bool {
        artist_tier >= self.required_recognition_tier
    }

    /// Spec A.5: calculate success score.
    pub fn calculate_success_score(
        &self,
        skills: &ProfessionalSkills,
        image: &ImageTags,
        personality: &PersonalitySpectrums,
        popularity: i32,
    ) -> i32 {
        let weighted_sum: f64 = self.skill_weights.iter().map(|(target, weight)| {
            let val = skill_value(skills, *target);
            val as f64 * weight
        }).sum();

        let image_mod = self.calculate_image_modifier(image);
        let personality_mod = self.calculate_personality_modifier(personality);
        let popularity_mod = 1.0 + (popularity - 50) as f64 / 200.0;

        (weighted_sum * image_mod * personality_mod * popularity_mod) as i32
    }

    /// Pay scales with popularity. Spec A.5 popularity_modifier.
    pub fn calculate_pay(&self, popularity: i32) -> i64 {
        let modifier = 1.0 + (popularity - 50) as f64 / 200.0;
        (self.base_pay as f64 * modifier) as i64
    }

    fn calculate_image_modifier(&self, image: &ImageTags) -> f64 {
        let mut modifier = 1.0;
        for (tag, threshold) in &self.ideal_image_tags {
            let val = tag.value_from(image);
            if val >= *threshold {
                modifier += if val >= threshold * 2 { 0.10 } else { 0.05 };
            }
        }
        for (tag, threshold) in &self.conflicting_image_tags {
            if tag.value_from(image) >= *threshold {
                modifier -= 0.10;
            }
        }
        modifier.clamp(0.80, 1.20)
    }

    fn calculate_personality_modifier(&self, personality: &PersonalitySpectrums) -> f64 {
        match &self.personality_preference {
            None => 1.0,
            Some((spectrum, preferred_min)) => {
                let val = personality.get(*spectrum);
                if val >= *preferred_min {
                    1.0 + (val - preferred_min).min(100) as f64 / 100.0 * 0.15
                } else {
                    let deficit = (preferred_min - val).min(100) as f64 / 100.0;
                    1.0 - deficit * 0.15
                }
            }
        }
    }
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
```

- [ ] **Step 4: Add `pub mod gig;` to lib.rs**

- [ ] **Step 5: Create `data/gigs/default_gigs.ron`** with 3-5 sample gigs

- [ ] **Step 6: Run tests**

Run: `cargo test -p stardom-core gig`
Expected: 4 tests PASS

- [ ] **Step 7: Commit**

```bash
git add crates/stardom-core/src/gig.rs crates/stardom-core/src/lib.rs data/gigs/
git commit -m "feat(core): add gig system with success score and pay calculation"
```

---

## Task 5: Scheduling — Apply Activity Effects to Artists

**Files:**
- Create: `crates/stardom-core/src/scheduling.rs`
- Modify: `crates/stardom-core/src/lib.rs` (add `pub mod scheduling;`)

This module contains the pure functions that apply activity effects to an artist. It does NOT handle the game loop — that's Task 6.

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::BaseAttributes;
    use crate::artist::Artist;
    use crate::types::ArtistId;

    fn make_test_artist() -> Artist {
        Artist::new(ArtistId(1), "Test".to_string(), 20, BaseAttributes::new(50, 50, 50, 50))
    }

    fn sample_training_def() -> TrainingDef {
        TrainingDef {
            id: TrainingId(1),
            name: "Vocal".to_string(),
            skill: SkillTarget::Vocal,
            tiers: vec![
                TrainingTier { cost: 8_000, base_gain: 40, stress_increase: 5, unlock_threshold: 0 },
            ],
            primary_attribute: PrimaryAttribute::Empathy,
            secondary_attribute: None,
        }
    }

    fn sample_job_def() -> JobDef {
        JobDef {
            id: JobId(1),
            name: "Street".to_string(),
            pay: 600,
            skill_gains: vec![(SkillTarget::Vocal, 15)],
            skill_losses: vec![(SkillTarget::Poise, 5)],
            recognition_gain: 3,
            stress_change: 3,
            required_recognition_tier: RecognitionTier::Unknown,
        }
    }

    #[test]
    fn apply_training_increases_skill_and_stress() {
        let mut artist = make_test_artist();
        let training = sample_training_def();
        let cost = apply_training(&mut artist, &training);
        assert_eq!(artist.skills.vocal, 40);
        assert_eq!(artist.stats.stress, 5);
        assert_eq!(cost, 8_000);
    }

    #[test]
    fn apply_job_increases_and_decreases_skills() {
        let mut artist = make_test_artist();
        artist.skills.poise = 100;
        let job = sample_job_def();
        let pay = apply_job(&mut artist, &job);
        assert_eq!(artist.skills.vocal, 15);
        assert_eq!(artist.skills.poise, 95);
        assert_eq!(artist.stats.recognition, 3);
        assert_eq!(artist.stats.stress, 3);
        assert_eq!(pay, 600);
    }

    #[test]
    fn apply_rest_reduces_stress() {
        let mut artist = make_test_artist();
        artist.stats.stress = 40;
        apply_rest(&mut artist);
        assert_eq!(artist.stats.stress, 20);
    }

    #[test]
    fn start_gig_locks_artist() {
        let mut artist = make_test_artist();
        let gig = GigDef {
            id: GigId(1),
            name: "Movie".to_string(),
            category: GigCategory::FilmTv,
            duration_weeks: 4,
            required_recognition_tier: RecognitionTier::Unknown,
            skill_weights: vec![],
            base_pay: 100_000,
            recognition_reward: 50,
            reputation_reward: 3,
            stress_cost: 15,
            ideal_image_tags: vec![],
            conflicting_image_tags: vec![],
            personality_preference: None,
            skill_gains: vec![(SkillTarget::Acting, 50)],
        };
        start_gig(&mut artist, &gig);
        assert_eq!(artist.locked_weeks, 4);
        assert!(artist.is_locked());
        assert_eq!(artist.current_activity, Activity::Gig);
    }

    #[test]
    fn complete_gig_rewards() {
        let mut artist = make_test_artist();
        let gig_def = GigDef {
            id: GigId(1),
            name: "Movie".to_string(),
            category: GigCategory::FilmTv,
            duration_weeks: 1,
            required_recognition_tier: RecognitionTier::Unknown,
            skill_weights: vec![(SkillTarget::Acting, 1.0)],
            base_pay: 100_000,
            recognition_reward: 50,
            reputation_reward: 3,
            stress_cost: 15,
            ideal_image_tags: vec![],
            conflicting_image_tags: vec![],
            personality_preference: None,
            skill_gains: vec![(SkillTarget::Acting, 50)],
        };
        artist.stats.popularity = 50;
        let pay = complete_gig(&mut artist, &gig_def);
        assert_eq!(artist.skills.acting, 50);
        assert_eq!(artist.stats.recognition, 50);
        assert_eq!(artist.stats.reputation, 3);
        assert_eq!(artist.stats.stress, 15);
        assert_eq!(pay, 100_000); // popularity=50 → modifier 1.0
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p stardom-core scheduling`
Expected: FAIL

- [ ] **Step 3: Implement scheduling module**

```rust
use crate::artist::Artist;
use crate::attribute::ProfessionalSkills;
use crate::gig::GigDef;
use crate::job::JobDef;
use crate::training::{SkillTarget, TrainingDef};
use crate::types::Activity;

const REST_STRESS_REDUCTION: i32 = 20;

/// Apply training to an artist. Returns cost.
pub fn apply_training(artist: &mut Artist, training: &TrainingDef) -> i64 {
    let current_skill = skill_value_mut(&artist.skills, training.skill);
    let tier_idx = training.best_tier_index(current_skill);
    let effect = training.calculate_effect(tier_idx, &artist.base_attributes, artist.stats.stress);

    apply_skill_gain(&mut artist.skills, training.skill, effect.skill_gain);
    artist.stats.stress = (artist.stats.stress + effect.stress_increase).min(100);
    artist.current_activity = Activity::Training;

    effect.cost
}

/// Apply part-time job to an artist. Returns pay earned.
pub fn apply_job(artist: &mut Artist, job: &JobDef) -> i64 {
    let effect = job.calculate_effect();
    for (target, gain) in &effect.skill_gains {
        apply_skill_gain(&mut artist.skills, *target, *gain);
    }
    for (target, loss) in &effect.skill_losses {
        apply_skill_loss(&mut artist.skills, *target, *loss);
    }
    artist.stats.add_recognition(effect.recognition_gain);
    artist.stats.stress = (artist.stats.stress + effect.stress_change).clamp(0, 100);
    artist.current_activity = Activity::PartTimeJob;

    effect.pay
}

/// Apply rest to an artist.
pub fn apply_rest(artist: &mut Artist) {
    artist.stats.stress = (artist.stats.stress - REST_STRESS_REDUCTION).max(0);
    artist.current_activity = Activity::Rest;
}

/// Start a multi-week gig. Locks the artist for duration_weeks.
pub fn start_gig(artist: &mut Artist, gig: &GigDef) {
    artist.locked_weeks = gig.duration_weeks;
    artist.current_activity = Activity::Gig;
}

/// Complete a gig and apply rewards. Returns pay earned.
pub fn complete_gig(artist: &mut Artist, gig: &GigDef) -> i64 {
    for (target, gain) in &gig.skill_gains {
        apply_skill_gain(&mut artist.skills, *target, *gain);
    }
    artist.stats.add_recognition(gig.recognition_reward);
    artist.stats.reputation = (artist.stats.reputation + gig.reputation_reward).clamp(-100, 100);
    artist.stats.stress = (artist.stats.stress + gig.stress_cost).min(100);

    gig.calculate_pay(artist.stats.popularity)
}

use crate::attribute::ProfessionalSkills;

fn get_skill_value(skills: &ProfessionalSkills, target: SkillTarget) -> i32 {
    match target {
        SkillTarget::Vocal => skills.vocal,
        SkillTarget::Acting => skills.acting,
        SkillTarget::Dance => skills.dance,
        SkillTarget::Poise => skills.poise,
        SkillTarget::Eloquence => skills.eloquence,
        SkillTarget::Creativity => skills.creativity,
    }
}

fn apply_skill_gain(skills: &mut ProfessionalSkills, target: SkillTarget, amount: i32) {
    let field = match target {
        SkillTarget::Vocal => &mut skills.vocal,
        SkillTarget::Acting => &mut skills.acting,
        SkillTarget::Dance => &mut skills.dance,
        SkillTarget::Poise => &mut skills.poise,
        SkillTarget::Eloquence => &mut skills.eloquence,
        SkillTarget::Creativity => &mut skills.creativity,
    };
    *field = (*field + amount).min(10_000);
}

fn apply_skill_loss(skills: &mut ProfessionalSkills, target: SkillTarget, amount: i32) {
    let field = match target {
        SkillTarget::Vocal => &mut skills.vocal,
        SkillTarget::Acting => &mut skills.acting,
        SkillTarget::Dance => &mut skills.dance,
        SkillTarget::Poise => &mut skills.poise,
        SkillTarget::Eloquence => &mut skills.eloquence,
        SkillTarget::Creativity => &mut skills.creativity,
    };
    *field = (*field - amount).max(0);
}
```

- [ ] **Step 4: Add `pub mod scheduling;` to lib.rs**

- [ ] **Step 5: Run tests**

Run: `cargo test -p stardom-core scheduling`
Expected: 5 tests PASS

- [ ] **Step 6: Commit**

```bash
git add crates/stardom-core/src/scheduling.rs crates/stardom-core/src/lib.rs
git commit -m "feat(core): add scheduling module with training, job, gig, and rest effects"
```

---

## Task 6: Wire Up Game Loop — AssignActivity & Weekly Processing

**Files:**
- Modify: `crates/stardom-core/src/game.rs`

This is the integration task. Extend `GameCommand` with `AssignActivity`, and update `advance_week` to process activities, decrement gig lock timers, complete gigs, and update financials.

- [ ] **Step 1: Write failing tests**

Add to `game.rs` tests:
```rust
use crate::training::{TrainingDef, TrainingTier, SkillTarget, PrimaryAttribute};
use crate::job::JobDef;
use crate::gig::{GigDef, GigCategory};
use crate::scheduling;
use crate::stats::RecognitionTier;

fn sample_training() -> TrainingDef {
    TrainingDef {
        id: TrainingId(1),
        name: "Vocal".to_string(),
        skill: SkillTarget::Vocal,
        tiers: vec![
            TrainingTier { cost: 8_000, base_gain: 40, stress_increase: 5, unlock_threshold: 0 },
        ],
        primary_attribute: PrimaryAttribute::Empathy,
        secondary_attribute: None,
    }
}

fn sample_job() -> JobDef {
    JobDef {
        id: JobId(1),
        name: "Street".to_string(),
        pay: 600,
        skill_gains: vec![(SkillTarget::Vocal, 15)],
        skill_losses: vec![],
        recognition_gain: 3,
        stress_change: 3,
        required_recognition_tier: RecognitionTier::Unknown,
    }
}

fn sample_gig() -> GigDef {
    GigDef {
        id: GigId(1),
        name: "Single".to_string(),
        category: GigCategory::Music,
        duration_weeks: 2,
        required_recognition_tier: RecognitionTier::Unknown,
        skill_weights: vec![(SkillTarget::Vocal, 1.0)],
        base_pay: 50_000,
        recognition_reward: 50,
        reputation_reward: 3,
        stress_cost: 10,
        ideal_image_tags: vec![],
        conflicting_image_tags: vec![],
        personality_preference: None,
        skill_gains: vec![(SkillTarget::Vocal, 30)],
    }
}

#[test]
fn assign_training_deducts_cost() {
    let mut game = default_game();
    game.artists.push(make_artist_with_popularity(0));
    let training = sample_training();
    game.process_command(GameCommand::AssignTraining { artist_index: 0, training: training });
    game.process_command(GameCommand::AdvanceWeek);
    // Training cost = 8000, starting balance = 1_000_000
    assert_eq!(game.company.balance, Money(1_000_000 - 8_000));
    assert_eq!(game.artists[0].skills.vocal, 40);
}

#[test]
fn assign_job_earns_money() {
    let mut game = default_game();
    game.artists.push(make_artist_with_popularity(0));
    let job = sample_job();
    game.process_command(GameCommand::AssignJob { artist_index: 0, job: job });
    game.process_command(GameCommand::AdvanceWeek);
    assert_eq!(game.company.balance, Money(1_000_000 + 600));
    assert_eq!(game.artists[0].skills.vocal, 15);
}

#[test]
fn assign_gig_locks_and_completes() {
    let mut game = default_game();
    game.artists.push(make_artist_with_popularity(50));
    let gig = sample_gig();
    game.process_command(GameCommand::AssignGig { artist_index: 0, gig: gig.clone() });

    // Week 1: gig started, locked_weeks decremented to 1
    game.process_command(GameCommand::AdvanceWeek);
    assert_eq!(game.artists[0].locked_weeks, 1);
    assert_eq!(game.artists[0].current_activity, Activity::Gig);
    // Popularity: was 50, Gig is public so no inactivity penalty, but base_decay=-2 → 48

    // Week 2: gig completes, rewards applied
    game.process_command(GameCommand::AdvanceWeek);
    assert_eq!(game.artists[0].locked_weeks, 0);
    assert_eq!(game.artists[0].skills.vocal, 30);
    // Pay at completion: base_pay=50000, popularity was 48
    // modifier = 1.0 + (48-50)/200 = 0.99 → pay = 49500
    assert_eq!(game.company.balance, Money(1_000_000 + 49_500));
}

#[test]
fn locked_artist_cannot_be_reassigned() {
    let mut game = default_game();
    game.artists.push(make_artist_with_popularity(0));
    let gig = sample_gig();
    game.process_command(GameCommand::AssignGig { artist_index: 0, gig: gig });

    game.process_command(GameCommand::AdvanceWeek);
    assert!(game.artists[0].is_locked());

    // Try to assign training while locked — should be ignored
    let training = sample_training();
    game.process_command(GameCommand::AssignTraining { artist_index: 0, training: training });
    game.process_command(GameCommand::AdvanceWeek);
    // Vocal should only have gig gains (30), not training gains
    // Gig completes this week, locked_weeks was 1 → 0
    assert_eq!(game.artists[0].skills.vocal, 30);
}

#[test]
fn rest_reduces_stress() {
    let mut game = default_game();
    let mut artist = make_artist_with_popularity(0);
    artist.stats.stress = 40;
    game.artists.push(artist);
    game.process_command(GameCommand::AssignRest { artist_index: 0 });
    game.process_command(GameCommand::AdvanceWeek);
    assert_eq!(game.artists[0].stats.stress, 20);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p stardom-core game`
Expected: FAIL

- [ ] **Step 3: Implement game loop integration**

Extend `GameCommand`:
```rust
#[derive(Debug, Clone)]
pub enum GameCommand {
    AdvanceWeek,
    AssignTraining { artist_index: usize, training: TrainingDef },
    AssignJob { artist_index: usize, job: JobDef },
    AssignGig { artist_index: usize, gig: GigDef },
    AssignRest { artist_index: usize },
}
```

Add to `GameState`:
```rust
pub pending_gigs: Vec<(usize, GigDef)>, // (artist_index, gig_def) for gigs being worked on
```

Update `process_command`:
```rust
fn process_command(&mut self, command: GameCommand) {
    if self.phase == GamePhase::GameOver { return; }
    match command {
        GameCommand::AdvanceWeek => self.advance_week(),
        GameCommand::AssignTraining { artist_index, training } => {
            if let Some(artist) = self.artists.get_mut(artist_index) {
                if !artist.is_locked() && artist.current_activity == Activity::Idle {
                    let cost = scheduling::apply_training(artist, &training);
                    self.company.spend(Money(cost));
                }
            }
        }
        GameCommand::AssignJob { artist_index, job } => {
            if let Some(artist) = self.artists.get_mut(artist_index) {
                if !artist.is_locked() && artist.current_activity == Activity::Idle {
                    let pay = scheduling::apply_job(artist, &job);
                    self.company.earn(Money(pay));
                }
            }
        }
        GameCommand::AssignGig { artist_index, gig } => {
            if let Some(artist) = self.artists.get_mut(artist_index) {
                if !artist.is_locked() && artist.current_activity == Activity::Idle {
                    scheduling::start_gig(artist, &gig);
                    self.pending_gigs.push((artist_index, gig));
                }
            }
        }
        GameCommand::AssignRest { artist_index } => {
            if let Some(artist) = self.artists.get_mut(artist_index) {
                if !artist.is_locked() && artist.current_activity == Activity::Idle {
                    scheduling::apply_rest(artist);
                }
            }
        }
    }
}
```

Update `advance_week` to handle gig progression:
```rust
fn advance_week(&mut self) {
    let was_last_week_of_year = self.calendar.week == WEEKS_PER_YEAR;
    self.calendar.advance_week();

    // Decrement gig lock timers and complete finished gigs
    let mut completed_gigs = Vec::new();
    for artist in &mut self.artists {
        if artist.locked_weeks > 0 {
            artist.locked_weeks -= 1;
            if artist.locked_weeks == 0 {
                completed_gigs.push(artist.id);
            }
        }
    }

    // Apply gig completion rewards (use mem::take to avoid borrow conflict)
    let mut pending = std::mem::take(&mut self.pending_gigs);
    let mut remaining = Vec::new();
    for (idx, gig_def) in pending.drain(..) {
        let is_complete = self
            .artists
            .get(idx)
            .is_some_and(|a| completed_gigs.contains(&a.id));
        if is_complete {
            let pay = scheduling::complete_gig(&mut self.artists[idx], &gig_def);
            self.company.earn(Money(pay));
        } else {
            remaining.push((idx, gig_def));
        }
    }
    self.pending_gigs = remaining;

    // Existing logic: aging, popularity decay, activity reset
    for artist in &mut self.artists {
        if was_last_week_of_year {
            artist.age += 1;
        }
        let active = artist.current_activity.is_public();
        artist.inactive_weeks = if active { 0 } else { artist.inactive_weeks + 1 };
        artist.stats.apply_weekly_popularity_decay(active, artist.inactive_weeks);
        if !artist.is_locked() {
            artist.current_activity = Activity::Idle;
        }
    }

    // Bankruptcy check — pending gigs count as pending income
    let has_pending_income = !self.pending_gigs.is_empty();
    self.company.update_bankruptcy_counter(has_pending_income);

    if self.company.is_bankrupt() {
        self.phase = GamePhase::GameOver;
    } else if self.phase == GamePhase::MainGame && self.calendar.is_goal_period_over() {
        self.phase = GamePhase::PostEnding;
    }
}
```

Also add the new imports to game.rs:
```rust
use crate::gig::GigDef;
use crate::job::JobDef;
use crate::scheduling;
use crate::training::TrainingDef;
use crate::types::{Activity, ArtistId, GigId, JobId, Money, TrainingId};
```

Initialize `pending_gigs` in `GameState::new`:
```rust
pending_gigs: Vec::new(),
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p stardom-core game`
Expected: all game tests PASS (old + new)

- [ ] **Step 5: Run full test suite**

Run: `cargo test -p stardom-core`
Expected: all tests PASS

- [ ] **Step 6: Commit**

```bash
git add crates/stardom-core/src/game.rs
git commit -m "feat(core): wire up AssignActivity commands with training, job, gig, rest processing"
```

---

## Task 7: Update Integration Test & Final Cleanup

**Files:**
- Modify: `crates/stardom-core/src/lib.rs` (update integration test)

- [ ] **Step 1: Update integration test to use activities**

Replace the existing `full_game_loop_smoke_test` with a richer version:
```rust
#[test]
fn full_game_loop_with_activities() {
    use crate::config::Settings;
    use crate::data_loader::load_artist_definition;
    use crate::game::{GameCommand, GamePhase, GameState};
    use crate::job::JobDef;
    use crate::stats::RecognitionTier;
    use crate::training::{TrainingDef, TrainingTier, SkillTarget, PrimaryAttribute};
    use crate::types::{JobId, Money, TrainingId};

    let mut game = GameState::new(Settings::default());

    let ron_str = r#"
        ArtistDefinition(
            id: ArtistId(1),
            name: "Luna Star",
            starting_age: 18,
            base_attributes: BaseAttributes(stamina: 60, intellect: 55, empathy: 70, charm: 80),
            personality: PersonalitySpectrums(social: 30, thinking: -20, action: 10, stance: -40),
            traits: InnerTraits(confidence: 55, rebellion: 25),
            image: ImageTags(pure: 60, sexy: 20, cool: 40, intellectual: 30, funny: 10, mysterious: 50),
        )
    "#;
    let def = load_artist_definition(ron_str).unwrap();
    game.artists.push(def.into_artist());

    let training = TrainingDef {
        id: TrainingId(1),
        name: "Vocal".to_string(),
        skill: SkillTarget::Vocal,
        tiers: vec![
            TrainingTier { cost: 8_000, base_gain: 40, stress_increase: 5, unlock_threshold: 0 },
        ],
        primary_attribute: PrimaryAttribute::Empathy,
        secondary_attribute: None,
    };

    let job = JobDef {
        id: JobId(1),
        name: "Street".to_string(),
        pay: 1_000,
        skill_gains: vec![(SkillTarget::Vocal, 10)],
        skill_losses: vec![],
        recognition_gain: 5,
        stress_change: 3,
        required_recognition_tier: RecognitionTier::Unknown,
    };

    // Alternate training and jobs for 10 weeks
    for i in 0..10 {
        if i % 3 == 2 {
            game.process_command(GameCommand::AssignRest { artist_index: 0 });
        } else if i % 3 == 0 {
            game.process_command(GameCommand::AssignTraining { artist_index: 0, training: training.clone() });
        } else {
            game.process_command(GameCommand::AssignJob { artist_index: 0, job: job.clone() });
        }
        game.process_command(GameCommand::AdvanceWeek);
    }

    // Artist should have gained skills from training and jobs
    assert!(game.artists[0].skills.vocal > 0);
    // Company should have spent on training and earned from jobs
    assert_ne!(game.company.balance, Money(1_000_000));
    // Stress should be managed due to rest weeks
    assert!(game.artists[0].stats.stress < 50);
}
```

- [ ] **Step 2: Run integration test**

Run: `cargo test -p stardom-core integration_tests`
Expected: PASS

- [ ] **Step 3: Run clippy and fmt**

Run: `cargo fmt --all && cargo clippy --workspace -- -D warnings`
Expected: clean

- [ ] **Step 4: Run full test suite**

Run: `cargo test --workspace`
Expected: all tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/stardom-core/src/lib.rs
git commit -m "test(core): update integration test with training, job, and rest activities"
```

---

## Phase 2 Completion Checklist

- [ ] `cargo test --workspace` — all tests pass
- [ ] `cargo clippy --workspace -- -D warnings` — clean
- [ ] `cargo fmt --check --all` — clean
- [ ] Training: tier selection, efficiency formula (Appendix A.1), costs deducted
- [ ] Part-time jobs: skill gains/losses, pay earned, recognition gained, tier-gated
- [ ] Gigs: success score (Appendix A.5), pay scales with popularity, multi-week locking
- [ ] Scheduling: activities applied to artists, locked artists can't be reassigned
- [ ] Game loop: AssignActivity commands, gig completion, financial flow
- [ ] Integration test covers multi-week scenario with mixed activities

---

## What Phase 3 Will Cover

> Not part of this plan. Listed for context only.

- Gig availability rotation (bi-weekly pool generation from data files)
- Award system (nomination, judging, ceremonies at calendar dates)
- PR crisis events (random event triggers, player choice resolution)
- Office upgrade system (purchase, bonuses)
