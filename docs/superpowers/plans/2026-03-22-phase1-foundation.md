# Phase 1: Foundation — Project Scaffolding & Core Data Models

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the two-layer Rust workspace (game-core lib + Bevy app), implement all attribute/stat data models with serialization, build the calendar system, and wire up a minimal game loop that advances time — all fully tested.

**Architecture:** Cargo workspace with two crates: `stardom-core` (pure Rust library, zero Bevy dependency) and `redo-stardom` (Bevy binary that depends on `stardom-core`). Phase 1 focuses almost entirely on `stardom-core` with only a skeleton Bevy app. Data files use RON for game content and TOML for configuration.

**Tech Stack:** Rust 1.94, Bevy 0.18, serde + ron + toml crates, cargo test

**Spec reference:** `docs/superpowers/specs/2026-03-22-redo-stardom-design.md`

---

## File Structure

```
redo-stardom/
├── Cargo.toml                          # workspace root
├── config/
│   └── settings.toml                   # game settings (configurable constants)
├── data/
│   └── artists/
│       └── sample_artist.ron           # sample artist definition for testing
├── crates/
│   ├── stardom-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # public API re-exports
│   │       ├── types.rs                # newtypes & ID types (ArtistId, GigId, etc.) + Activity enum
│   │       ├── attribute.rs            # base attributes, professional skills, inner traits
│   │       ├── persona.rs              # personality spectrums + image tags
│   │       ├── stats.rs                # auxiliary stats (recognition, reputation, popularity, stress)
│   │       ├── artist.rs               # Artist struct combining all attribute layers
│   │       ├── company.rs              # CompanyState (money, office tier, roster)
│   │       ├── calendar.rs             # Calendar, Week, Date, time advancement
│   │       ├── config.rs               # Settings loaded from TOML
│   │       ├── game.rs                 # GameState, GameCommand, top-level game loop
│   │       └── data_loader.rs          # RON/TOML file loading and validation
│   └── redo-stardom/
│       ├── Cargo.toml
│       └── src/
│           └── main.rs                 # minimal Bevy app skeleton
└── docs/
    └── ...
```

---

## Task 1: Workspace & Crate Scaffolding

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/stardom-core/Cargo.toml`
- Create: `crates/stardom-core/src/lib.rs`
- Create: `crates/redo-stardom/Cargo.toml`
- Create: `crates/redo-stardom/src/main.rs`

- [ ] **Step 1: Create workspace root Cargo.toml**

```toml
[workspace]
resolver = "2"
members = ["crates/stardom-core", "crates/redo-stardom"]
```

- [ ] **Step 2: Create stardom-core crate**

`crates/stardom-core/Cargo.toml`:
```toml
[package]
name = "stardom-core"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1", features = ["derive"] }
ron = "0.10"
toml = "0.8"
rand = "0.9"
```

`crates/stardom-core/src/lib.rs`:
```rust
pub mod types;
pub mod attribute;
pub mod persona;
pub mod stats;
pub mod artist;
pub mod company;
pub mod calendar;
pub mod config;
pub mod game;
pub mod data_loader;
```

Create empty module files for each module:
```rust
// each file starts as empty or with a placeholder comment
```

- [ ] **Step 3: Create redo-stardom (Bevy app) crate**

`crates/redo-stardom/Cargo.toml`:
```toml
[package]
name = "redo-stardom"
version = "0.1.0"
edition = "2024"

[dependencies]
stardom-core = { path = "../stardom-core" }
bevy = "0.18"
```

`crates/redo-stardom/src/main.rs`:
```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "REDÓ Stardom".to_string(),
                resolution: WindowResolution::new(800.0, 600.0),
                ..default()
            }),
            ..default()
        }))
        .run();
}
```

- [ ] **Step 4: Add .gitignore entries for Rust**

Append to existing `.gitignore`:
```
# Rust
/target
```

- [ ] **Step 5: Verify workspace builds**

Run: `cargo build --workspace`
Expected: compiles with no errors

- [ ] **Step 6: Verify tests run (empty)**

Run: `cargo test --workspace`
Expected: 0 tests, no errors

- [ ] **Step 7: Commit**

```bash
git add crates/ Cargo.toml .gitignore
git commit -m "feat: scaffold workspace with stardom-core lib and redo-stardom app"
```

---

## Task 2: ID Types & Core Newtypes

**Files:**
- Create: `crates/stardom-core/src/types.rs`
- Test: inline `#[cfg(test)]` in `types.rs`

- [ ] **Step 1: Write failing tests for ID types**

In `crates/stardom-core/src/types.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artist_id_equality() {
        let a = ArtistId(1);
        let b = ArtistId(1);
        assert_eq!(a, b);
    }

    #[test]
    fn artist_id_serialization() {
        let id = ArtistId(42);
        let serialized = ron::to_string(&id).unwrap();
        let deserialized: ArtistId = ron::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn money_arithmetic() {
        let a = Money(10_000);
        let b = Money(3_000);
        assert_eq!(a + b, Money(13_000));
        assert_eq!(a - b, Money(7_000));
    }

    #[test]
    fn money_can_be_negative() {
        let a = Money(1_000);
        let b = Money(5_000);
        assert_eq!(a - b, Money(-4_000));
    }

    #[test]
    fn activity_is_public() {
        assert!(Activity::Gig.is_public());
        assert!(Activity::PartTimeJob.is_public());
        assert!(!Activity::Training.is_public());
        assert!(!Activity::Rest.is_public());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p stardom-core types`
Expected: FAIL — types not defined

- [ ] **Step 3: Implement ID types and Money**

```rust
use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtistId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GigId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OutfitId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CrisisId(pub u32);

/// Game currency. Stored as i64 to allow negative balances (debt).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Money(pub i64);

impl Add for Money {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { Money(self.0 + rhs.0) }
}

impl Sub for Money {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { Money(self.0 - rhs.0) }
}

/// What an artist is doing this week.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activity {
    Training,
    PartTimeJob,
    Gig,
    Rest,
    Idle, // no assignment
}

impl Activity {
    /// Whether this activity counts as public exposure (prevents popularity inactivity penalty).
    pub fn is_public(&self) -> bool {
        matches!(self, Activity::Gig | Activity::PartTimeJob)
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p stardom-core types`
Expected: 4 tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/stardom-core/src/types.rs
git commit -m "feat(core): add ID newtypes and Money arithmetic"
```

---

## Task 3: Base Attributes (4-stat diamond)

**Files:**
- Create: `crates/stardom-core/src/attribute.rs`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_attributes_clamp_to_range() {
        let mut attrs = BaseAttributes::new(50, 60, 70, 80);
        attrs.stamina = 150; // over max
        attrs.clamp();
        assert_eq!(attrs.stamina, 100);
    }

    #[test]
    fn base_attributes_default_values() {
        let attrs = BaseAttributes::default();
        assert_eq!(attrs.stamina, 50);
        assert_eq!(attrs.intellect, 50);
        assert_eq!(attrs.empathy, 50);
        assert_eq!(attrs.charm, 50);
    }

    #[test]
    fn professional_skills_start_at_zero() {
        let skills = ProfessionalSkills::default();
        assert_eq!(skills.vocal, 0);
        assert_eq!(skills.acting, 0);
    }

    #[test]
    fn professional_skills_clamp_to_max() {
        let mut skills = ProfessionalSkills::default();
        skills.vocal = 12_000;
        skills.clamp();
        assert_eq!(skills.vocal, 10_000);
    }

    #[test]
    fn inner_traits_default() {
        let traits = InnerTraits::default();
        assert_eq!(traits.confidence, 50);
        assert_eq!(traits.rebellion, 30);
    }

    #[test]
    fn inner_traits_clamp() {
        let mut traits = InnerTraits { confidence: 120, rebellion: -5 };
        traits.clamp();
        assert_eq!(traits.confidence, 100);
        assert_eq!(traits.rebellion, 0);
    }

    #[test]
    fn serialization_roundtrip() {
        let attrs = BaseAttributes::new(40, 55, 70, 85);
        let s = ron::to_string(&attrs).unwrap();
        let d: BaseAttributes = ron::from_str(&s).unwrap();
        assert_eq!(attrs, d);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p stardom-core attribute`
Expected: FAIL

- [ ] **Step 3: Implement attribute types**

```rust
use serde::{Deserialize, Serialize};

pub const BASE_ATTR_MIN: i32 = 1;
pub const BASE_ATTR_MAX: i32 = 100;
pub const SKILL_MIN: i32 = 0;
pub const SKILL_MAX: i32 = 10_000;
pub const TRAIT_MIN: i32 = 0;
pub const TRAIT_MAX: i32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseAttributes {
    pub stamina: i32,
    pub intellect: i32,
    pub empathy: i32,
    pub charm: i32,
}

impl Default for BaseAttributes {
    fn default() -> Self {
        Self { stamina: 50, intellect: 50, empathy: 50, charm: 50 }
    }
}

impl BaseAttributes {
    pub fn new(stamina: i32, intellect: i32, empathy: i32, charm: i32) -> Self {
        let mut a = Self { stamina, intellect, empathy, charm };
        a.clamp();
        a
    }

    pub fn clamp(&mut self) {
        self.stamina = self.stamina.clamp(BASE_ATTR_MIN, BASE_ATTR_MAX);
        self.intellect = self.intellect.clamp(BASE_ATTR_MIN, BASE_ATTR_MAX);
        self.empathy = self.empathy.clamp(BASE_ATTR_MIN, BASE_ATTR_MAX);
        self.charm = self.charm.clamp(BASE_ATTR_MIN, BASE_ATTR_MAX);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProfessionalSkills {
    pub vocal: i32,
    pub acting: i32,
    pub dance: i32,
    pub poise: i32,
    pub eloquence: i32,
    pub creativity: i32,
}

impl ProfessionalSkills {
    pub fn clamp(&mut self) {
        self.vocal = self.vocal.clamp(SKILL_MIN, SKILL_MAX);
        self.acting = self.acting.clamp(SKILL_MIN, SKILL_MAX);
        self.dance = self.dance.clamp(SKILL_MIN, SKILL_MAX);
        self.poise = self.poise.clamp(SKILL_MIN, SKILL_MAX);
        self.eloquence = self.eloquence.clamp(SKILL_MIN, SKILL_MAX);
        self.creativity = self.creativity.clamp(SKILL_MIN, SKILL_MAX);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InnerTraits {
    pub confidence: i32,
    pub rebellion: i32,
}

impl Default for InnerTraits {
    fn default() -> Self {
        Self { confidence: 50, rebellion: 30 }
    }
}

impl InnerTraits {
    pub fn clamp(&mut self) {
        self.confidence = self.confidence.clamp(TRAIT_MIN, TRAIT_MAX);
        self.rebellion = self.rebellion.clamp(TRAIT_MIN, TRAIT_MAX);
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p stardom-core attribute`
Expected: 7 tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/stardom-core/src/attribute.rs
git commit -m "feat(core): add BaseAttributes, ProfessionalSkills, InnerTraits"
```

---

## Task 4: Personality Spectrums & Image Tags

**Files:**
- Create: `crates/stardom-core/src/persona.rs`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spectrum_clamp_to_range() {
        let mut p = PersonalitySpectrums::default();
        p.social = 150;
        p.clamp();
        assert_eq!(p.social, 100);
        p.social = -200;
        p.clamp();
        assert_eq!(p.social, -100);
    }

    #[test]
    fn spectrum_modifier_at_extremes() {
        let p = PersonalitySpectrums { social: 100, thinking: 0, action: -100, stance: 50 };
        assert!((p.modifier(Spectrum::Social) - 0.15).abs() < 0.001);
        assert!((p.modifier(Spectrum::Thinking) - 0.0).abs() < 0.001);
        assert!((p.modifier(Spectrum::Action) - 0.15).abs() < 0.001);
        assert!((p.modifier(Spectrum::Stance) - 0.075).abs() < 0.001);
    }

    #[test]
    fn image_tags_independent() {
        let mut tags = ImageTags::default();
        tags.pure = 80;
        tags.sexy = 70;
        tags.clamp();
        assert_eq!(tags.pure, 80);
        assert_eq!(tags.sexy, 70);
    }

    #[test]
    fn image_tags_clamp() {
        let mut tags = ImageTags::default();
        tags.cool = 120;
        tags.clamp();
        assert_eq!(tags.cool, 100);
    }

    #[test]
    fn serialization_roundtrip() {
        let p = PersonalitySpectrums { social: -30, thinking: 45, action: 0, stance: -80 };
        let s = ron::to_string(&p).unwrap();
        let d: PersonalitySpectrums = ron::from_str(&s).unwrap();
        assert_eq!(p, d);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p stardom-core persona`
Expected: FAIL

- [ ] **Step 3: Implement persona types**

```rust
use serde::{Deserialize, Serialize};

pub const SPECTRUM_MIN: i32 = -100;
pub const SPECTRUM_MAX: i32 = 100;
pub const IMAGE_TAG_MIN: i32 = 0;
pub const IMAGE_TAG_MAX: i32 = 100;
const MAX_SPECTRUM_BONUS: f64 = 0.15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spectrum {
    Social,
    Thinking,
    Action,
    Stance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonalitySpectrums {
    /// -100 (Introvert) to +100 (Extrovert)
    pub social: i32,
    /// -100 (Intuitive) to +100 (Logical)
    pub thinking: i32,
    /// -100 (Cautious) to +100 (Adventurous)
    pub action: i32,
    /// -100 (Easygoing) to +100 (Competitive)
    pub stance: i32,
}

impl Default for PersonalitySpectrums {
    fn default() -> Self {
        Self { social: 0, thinking: 0, action: 0, stance: 0 }
    }
}

impl PersonalitySpectrums {
    pub fn clamp(&mut self) {
        self.social = self.social.clamp(SPECTRUM_MIN, SPECTRUM_MAX);
        self.thinking = self.thinking.clamp(SPECTRUM_MIN, SPECTRUM_MAX);
        self.action = self.action.clamp(SPECTRUM_MIN, SPECTRUM_MAX);
        self.stance = self.stance.clamp(SPECTRUM_MIN, SPECTRUM_MAX);
    }

    /// Returns the absolute modifier strength (0.0 to MAX_SPECTRUM_BONUS)
    /// for the given spectrum. Direction (left/right) is determined by sign.
    pub fn modifier(&self, spectrum: Spectrum) -> f64 {
        let value = match spectrum {
            Spectrum::Social => self.social,
            Spectrum::Thinking => self.thinking,
            Spectrum::Action => self.action,
            Spectrum::Stance => self.stance,
        };
        (value.abs() as f64 / 100.0) * MAX_SPECTRUM_BONUS
    }

    pub fn get(&self, spectrum: Spectrum) -> i32 {
        match spectrum {
            Spectrum::Social => self.social,
            Spectrum::Thinking => self.thinking,
            Spectrum::Action => self.action,
            Spectrum::Stance => self.stance,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ImageTags {
    pub pure: i32,
    pub sexy: i32,
    pub cool: i32,
    pub intellectual: i32,
    pub funny: i32,
    pub mysterious: i32,
}

impl ImageTags {
    pub fn clamp(&mut self) {
        self.pure = self.pure.clamp(IMAGE_TAG_MIN, IMAGE_TAG_MAX);
        self.sexy = self.sexy.clamp(IMAGE_TAG_MIN, IMAGE_TAG_MAX);
        self.cool = self.cool.clamp(IMAGE_TAG_MIN, IMAGE_TAG_MAX);
        self.intellectual = self.intellectual.clamp(IMAGE_TAG_MIN, IMAGE_TAG_MAX);
        self.funny = self.funny.clamp(IMAGE_TAG_MIN, IMAGE_TAG_MAX);
        self.mysterious = self.mysterious.clamp(IMAGE_TAG_MIN, IMAGE_TAG_MAX);
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p stardom-core persona`
Expected: 5 tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/stardom-core/src/persona.rs
git commit -m "feat(core): add PersonalitySpectrums and ImageTags"
```

---

## Task 5: Auxiliary Stats

**Files:**
- Create: `crates/stardom-core/src/stats.rs`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognition_only_increases() {
        let mut s = AuxiliaryStats::default();
        s.add_recognition(100);
        assert_eq!(s.recognition, 100);
        s.add_recognition(-50); // should be ignored
        assert_eq!(s.recognition, 100);
    }

    #[test]
    fn recognition_tier_lookup() {
        assert_eq!(RecognitionTier::from_value(0), RecognitionTier::Unknown);
        assert_eq!(RecognitionTier::from_value(99), RecognitionTier::Unknown);
        assert_eq!(RecognitionTier::from_value(100), RecognitionTier::Newcomer);
        assert_eq!(RecognitionTier::from_value(500), RecognitionTier::Rising);
        assert_eq!(RecognitionTier::from_value(2_000), RecognitionTier::Established);
        assert_eq!(RecognitionTier::from_value(5_000), RecognitionTier::Star);
        assert_eq!(RecognitionTier::from_value(15_000), RecognitionTier::Superstar);
    }

    #[test]
    fn reputation_clamp() {
        let mut s = AuxiliaryStats::default();
        s.reputation = 150;
        s.clamp();
        assert_eq!(s.reputation, 100);
        s.reputation = -150;
        s.clamp();
        assert_eq!(s.reputation, -100);
    }

    #[test]
    fn popularity_decay_active() {
        let mut s = AuxiliaryStats { popularity: 50, ..Default::default() };
        s.apply_weekly_popularity_decay(true, 0);
        assert_eq!(s.popularity, 48); // base_decay = -2
    }

    #[test]
    fn popularity_decay_inactive_3_weeks() {
        let mut s = AuxiliaryStats { popularity: 50, ..Default::default() };
        s.apply_weekly_popularity_decay(false, 3);
        assert_eq!(s.popularity, 42); // -2 base + -6 inactivity
    }

    #[test]
    fn popularity_floor_at_zero() {
        let mut s = AuxiliaryStats { popularity: 3, ..Default::default() };
        s.apply_weekly_popularity_decay(false, 5);
        assert_eq!(s.popularity, 0);
    }

    #[test]
    fn stress_condition_modifier() {
        assert!((stress_condition_modifier(20) - 1.0).abs() < 0.001);
        assert!((stress_condition_modifier(45) - 0.85).abs() < 0.001);
        assert!((stress_condition_modifier(70) - 0.65).abs() < 0.001);
        assert!((stress_condition_modifier(90) - 0.40).abs() < 0.001);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p stardom-core stats`
Expected: FAIL

- [ ] **Step 3: Implement auxiliary stats**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecognitionTier {
    Unknown,     // 0–99
    Newcomer,    // 100–499
    Rising,      // 500–1,999
    Established, // 2,000–4,999
    Star,        // 5,000–14,999
    Superstar,   // 15,000+
}

impl RecognitionTier {
    pub fn from_value(recognition: i64) -> Self {
        match recognition {
            0..100 => Self::Unknown,
            100..500 => Self::Newcomer,
            500..2_000 => Self::Rising,
            2_000..5_000 => Self::Established,
            5_000..15_000 => Self::Star,
            _ => Self::Superstar,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuxiliaryStats {
    /// 0 → ∞, only increases
    pub recognition: i64,
    /// -100 to +100
    pub reputation: i32,
    /// 0 to 100, decays weekly
    pub popularity: i32,
    /// 0 to 100
    pub stress: i32,
}

impl Default for AuxiliaryStats {
    fn default() -> Self {
        Self { recognition: 0, reputation: 0, popularity: 0, stress: 0 }
    }
}

impl AuxiliaryStats {
    pub fn add_recognition(&mut self, amount: i64) {
        if amount > 0 {
            self.recognition += amount;
        }
    }

    pub fn recognition_tier(&self) -> RecognitionTier {
        RecognitionTier::from_value(self.recognition)
    }

    pub fn clamp(&mut self) {
        self.reputation = self.reputation.clamp(-100, 100);
        self.popularity = self.popularity.clamp(0, 100);
        self.stress = self.stress.clamp(0, 100);
        // recognition has no upper bound and no lower bound below 0
        if self.recognition < 0 {
            self.recognition = 0;
        }
    }

    /// Apply weekly popularity decay.
    /// `active_this_week`: whether the artist had any public activity.
    /// `consecutive_inactive_weeks`: how many consecutive weeks without activity (0 if active).
    pub fn apply_weekly_popularity_decay(
        &mut self,
        active_this_week: bool,
        consecutive_inactive_weeks: u32,
    ) {
        let base_decay = 2;
        let inactivity_penalty = if active_this_week {
            0
        } else {
            match consecutive_inactive_weeks {
                0 => 0,
                1 => 2,
                2 => 4,
                _ => 6, // caps at 3+
            }
        };
        self.popularity = (self.popularity - base_decay - inactivity_penalty as i32).max(0);
    }
}

/// Returns the training/activity efficiency modifier based on stress level.
/// Spec A.4: 0-30 → 1.0, 31-60 → 0.85, 61-80 → 0.65, 81-100 → 0.40
pub fn stress_condition_modifier(stress: i32) -> f64 {
    match stress {
        0..=30 => 1.0,
        31..=60 => 0.85,
        61..=80 => 0.65,
        _ => 0.40,
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p stardom-core stats`
Expected: 7 tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/stardom-core/src/stats.rs
git commit -m "feat(core): add AuxiliaryStats with recognition tiers, popularity decay, stress modifiers"
```

---

## Task 6: Artist Struct (Combining All Layers)

**Files:**
- Create: `crates/stardom-core/src/artist.rs`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::BaseAttributes;

    #[test]
    fn artist_creation() {
        let artist = Artist::new(
            ArtistId(1),
            "Test Star".to_string(),
            18,
            BaseAttributes::new(60, 50, 70, 80),
        );
        assert_eq!(artist.id, ArtistId(1));
        assert_eq!(artist.name, "Test Star");
        assert_eq!(artist.age, 18);
        assert_eq!(artist.base_attributes.stamina, 60);
        assert_eq!(artist.skills, ProfessionalSkills::default());
    }

    #[test]
    fn artist_serialization_roundtrip() {
        let artist = Artist::new(
            ArtistId(1),
            "Test Star".to_string(),
            20,
            BaseAttributes::new(50, 50, 50, 50),
        );
        let s = ron::to_string(&artist).unwrap();
        let d: Artist = ron::from_str(&s).unwrap();
        assert_eq!(artist.id, d.id);
        assert_eq!(artist.name, d.name);
    }

    #[test]
    fn artist_is_retired_at_age_limit() {
        let mut artist = Artist::new(
            ArtistId(1),
            "Old Star".to_string(),
            39,
            BaseAttributes::default(),
        );
        assert!(!artist.is_retired(40));
        artist.age = 40;
        assert!(artist.is_retired(40));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p stardom-core artist`
Expected: FAIL

- [ ] **Step 3: Implement Artist**

```rust
use serde::{Deserialize, Serialize};

use crate::attribute::{BaseAttributes, InnerTraits, ProfessionalSkills};
use crate::persona::{ImageTags, PersonalitySpectrums};
use crate::stats::AuxiliaryStats;
use crate::types::{Activity, ArtistId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub id: ArtistId,
    pub name: String,
    pub age: u32,
    pub base_attributes: BaseAttributes,
    pub skills: ProfessionalSkills,
    pub traits: InnerTraits,
    pub personality: PersonalitySpectrums,
    pub image: ImageTags,
    pub stats: AuxiliaryStats,
    /// What the artist is doing this week
    pub current_activity: Activity,
    /// Consecutive weeks without public activity (for popularity decay)
    pub inactive_weeks: u32,
}

impl Artist {
    pub fn new(id: ArtistId, name: String, age: u32, base_attributes: BaseAttributes) -> Self {
        Self {
            id,
            name,
            age,
            base_attributes,
            skills: ProfessionalSkills::default(),
            traits: InnerTraits::default(),
            personality: PersonalitySpectrums::default(),
            image: ImageTags::default(),
            stats: AuxiliaryStats::default(),
            current_activity: Activity::Idle,
            inactive_weeks: 0,
        }
    }

    pub fn is_retired(&self, retirement_age: u32) -> bool {
        self.age >= retirement_age
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p stardom-core artist`
Expected: 3 tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/stardom-core/src/artist.rs
git commit -m "feat(core): add Artist struct combining all attribute layers"
```

---

## Task 7: Calendar System

**Files:**
- Create: `crates/stardom-core/src/calendar.rs`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_calendar_starts_at_week_1_year_1() {
        let cal = Calendar::new(3);
        assert_eq!(cal.year, 1);
        assert_eq!(cal.week, 1);
        assert_eq!(cal.total_weeks_elapsed, 0);
        assert_eq!(cal.goal_years, 3);
    }

    #[test]
    fn advance_week() {
        let mut cal = Calendar::new(3);
        cal.advance_week();
        assert_eq!(cal.week, 2);
        assert_eq!(cal.total_weeks_elapsed, 1);
    }

    #[test]
    fn year_rolls_over_at_week_52() {
        let mut cal = Calendar::new(3);
        for _ in 0..52 {
            cal.advance_week();
        }
        assert_eq!(cal.year, 2);
        assert_eq!(cal.week, 1);
        assert_eq!(cal.total_weeks_elapsed, 52);
    }

    #[test]
    fn goal_reached_after_3_years() {
        let mut cal = Calendar::new(3);
        assert!(!cal.is_goal_period_over());
        for _ in 0..(52 * 3) {
            cal.advance_week();
        }
        assert!(cal.is_goal_period_over());
    }

    #[test]
    fn is_biweekly_rotation() {
        let cal_week1 = Calendar { year: 1, week: 1, total_weeks_elapsed: 0, goal_years: 3 };
        let cal_week2 = Calendar { year: 1, week: 2, total_weeks_elapsed: 1, goal_years: 3 };
        // Odd weeks = rotation A, even weeks = rotation B
        assert!(cal_week1.is_rotation_a());
        assert!(!cal_week2.is_rotation_a());
    }

    #[test]
    fn approximate_month() {
        let cal = Calendar { year: 1, week: 1, total_weeks_elapsed: 0, goal_years: 3 };
        assert_eq!(cal.approximate_month(), 1); // January
        let cal = Calendar { year: 1, week: 36, total_weeks_elapsed: 35, goal_years: 3 };
        assert_eq!(cal.approximate_month(), 9); // ~September
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p stardom-core calendar`
Expected: FAIL

- [ ] **Step 3: Implement Calendar**

```rust
use serde::{Deserialize, Serialize};

pub const WEEKS_PER_YEAR: u32 = 52;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Calendar {
    pub year: u32,
    pub week: u32, // 1-based, 1..=52
    pub total_weeks_elapsed: u32,
    pub goal_years: u32,
}

impl Calendar {
    pub fn new(goal_years: u32) -> Self {
        Self {
            year: 1,
            week: 1,
            total_weeks_elapsed: 0,
            goal_years,
        }
    }

    pub fn advance_week(&mut self) {
        self.total_weeks_elapsed += 1;
        self.week += 1;
        if self.week > WEEKS_PER_YEAR {
            self.week = 1;
            self.year += 1;
        }
    }

    pub fn is_goal_period_over(&self) -> bool {
        self.total_weeks_elapsed >= self.goal_years * WEEKS_PER_YEAR
    }

    /// Bi-weekly gig rotation: odd weeks = rotation A, even weeks = rotation B
    pub fn is_rotation_a(&self) -> bool {
        self.week % 2 == 1
    }

    /// Approximate month (1-12) from week number
    pub fn approximate_month(&self) -> u32 {
        ((self.week - 1) * 12 / WEEKS_PER_YEAR) + 1
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p stardom-core calendar`
Expected: 6 tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/stardom-core/src/calendar.rs
git commit -m "feat(core): add Calendar with week advancement and bi-weekly rotation"
```

---

## Task 8: Company State

**Files:**
- Create: `crates/stardom-core/src/company.rs`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Money;

    #[test]
    fn new_company() {
        let company = CompanyState::new(Money(1_000_000), 3);
        assert_eq!(company.balance, Money(1_000_000));
        assert_eq!(company.office_tier, OfficeTier::Starter);
        assert_eq!(company.max_artists, 3);
        assert_eq!(company.consecutive_negative_weeks, 0);
    }

    #[test]
    fn spend_money() {
        let mut company = CompanyState::new(Money(100_000), 3);
        company.spend(Money(50_000));
        assert_eq!(company.balance, Money(50_000));
    }

    #[test]
    fn spend_can_go_negative() {
        let mut company = CompanyState::new(Money(10_000), 3);
        company.spend(Money(20_000));
        assert_eq!(company.balance, Money(-10_000));
    }

    #[test]
    fn bankruptcy_check_resets_on_positive() {
        let mut company = CompanyState::new(Money(-5_000), 3);
        company.consecutive_negative_weeks = 3;
        company.update_bankruptcy_counter(true);
        assert_eq!(company.consecutive_negative_weeks, 3); // still negative balance
        company.balance = Money(100);
        company.update_bankruptcy_counter(false);
        assert_eq!(company.consecutive_negative_weeks, 0);
    }

    #[test]
    fn bankruptcy_triggers_at_4_weeks() {
        let mut company = CompanyState::new(Money(-5_000), 3);
        company.consecutive_negative_weeks = 3;
        assert!(!company.is_bankrupt());
        company.update_bankruptcy_counter(false); // no pending income
        assert!(company.is_bankrupt());
    }

    #[test]
    fn bankruptcy_paused_by_pending_income() {
        let mut company = CompanyState::new(Money(-5_000), 3);
        company.consecutive_negative_weeks = 3;
        company.update_bankruptcy_counter(true); // has pending income
        assert_eq!(company.consecutive_negative_weeks, 3); // counter frozen, not incremented
        assert!(!company.is_bankrupt());
    }

    #[test]
    fn office_tier_ordering() {
        assert!(OfficeTier::Starter < OfficeTier::Standard);
        assert!(OfficeTier::Standard < OfficeTier::Premium);
        assert!(OfficeTier::Premium < OfficeTier::Luxury);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p stardom-core company`
Expected: FAIL

- [ ] **Step 3: Implement CompanyState**

```rust
use serde::{Deserialize, Serialize};
use crate::types::Money;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficeTier {
    Starter,
    Standard,
    Premium,
    Luxury,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyState {
    pub balance: Money,
    pub office_tier: OfficeTier,
    pub max_artists: u32,
    pub consecutive_negative_weeks: u32,
}

impl CompanyState {
    pub fn new(starting_balance: Money, max_artists: u32) -> Self {
        Self {
            balance: starting_balance,
            office_tier: OfficeTier::Starter,
            max_artists,
            consecutive_negative_weeks: 0,
        }
    }

    /// Deduct money. Debt is allowed per spec A.13 — balance can go negative.
    pub fn spend(&mut self, amount: Money) {
        self.balance = self.balance - amount;
    }

    pub fn earn(&mut self, amount: Money) {
        self.balance = self.balance + amount;
    }

    /// Update bankruptcy counter at end of week.
    /// `has_pending_income`: true if any artist has a gig completing within 2 weeks.
    pub fn update_bankruptcy_counter(&mut self, has_pending_income: bool) {
        if self.balance.0 >= 0 {
            self.consecutive_negative_weeks = 0;
        } else if !has_pending_income {
            self.consecutive_negative_weeks += 1;
        }
        // If has_pending_income and balance < 0, counter is frozen (not incremented)
    }

    pub fn is_bankrupt(&self) -> bool {
        self.balance.0 < 0 && self.consecutive_negative_weeks >= 4
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p stardom-core company`
Expected: 7 tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/stardom-core/src/company.rs
git commit -m "feat(core): add CompanyState with bankruptcy model"
```

---

## Task 9: Config (TOML Settings)

**Files:**
- Create: `crates/stardom-core/src/config.rs`
- Create: `config/settings.toml`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings() {
        let s = Settings::default();
        assert_eq!(s.goal_years, 3);
        assert_eq!(s.retirement_age, 40);
        assert_eq!(s.max_artists, 3);
        assert_eq!(s.starting_balance, 1_000_000);
    }

    #[test]
    fn load_from_toml_string() {
        let toml_str = r#"
            goal_years = 5
            retirement_age = 35
            max_artists = 4
            starting_balance = 2_000_000
        "#;
        let s: Settings = toml::from_str(toml_str).unwrap();
        assert_eq!(s.goal_years, 5);
        assert_eq!(s.retirement_age, 35);
        assert_eq!(s.max_artists, 4);
        assert_eq!(s.starting_balance, 2_000_000);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p stardom-core config`
Expected: FAIL

- [ ] **Step 3: Implement Settings**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub goal_years: u32,
    pub retirement_age: u32,
    pub max_artists: u32,
    pub starting_balance: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            goal_years: 3,
            retirement_age: 40,
            max_artists: 3,
            starting_balance: 1_000_000,
        }
    }
}

impl Settings {
    pub fn load_from_str(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }
}
```

- [ ] **Step 4: Create default settings.toml**

`config/settings.toml`:
```toml
# REDÓ Stardom — Game Settings
# These values can be modified by players/modders.

goal_years = 3
retirement_age = 40
max_artists = 3
starting_balance = 1_000_000
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p stardom-core config`
Expected: 2 tests PASS

- [ ] **Step 6: Commit**

```bash
git add crates/stardom-core/src/config.rs config/
git commit -m "feat(core): add Settings with TOML loading"
```

---

## Task 10: Data Loader (RON Artist Definitions)

**Files:**
- Create: `crates/stardom-core/src/data_loader.rs`
- Create: `data/artists/sample_artist.ron`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_artist_definition_from_ron() {
        let ron_str = r#"
            ArtistDefinition(
                id: ArtistId(1),
                name: "Luna Star",
                starting_age: 18,
                base_attributes: BaseAttributes(
                    stamina: 60,
                    intellect: 55,
                    empathy: 70,
                    charm: 80,
                ),
                personality: PersonalitySpectrums(
                    social: 30,
                    thinking: -20,
                    action: 10,
                    stance: -40,
                ),
                traits: InnerTraits(
                    confidence: 55,
                    rebellion: 25,
                ),
                image: ImageTags(
                    pure: 60,
                    sexy: 20,
                    cool: 40,
                    intellectual: 30,
                    funny: 10,
                    mysterious: 50,
                ),
            )
        "#;
        let def: ArtistDefinition = ron::from_str(ron_str).unwrap();
        assert_eq!(def.name, "Luna Star");
        assert_eq!(def.starting_age, 18);
        assert_eq!(def.base_attributes.charm, 80);
        assert_eq!(def.personality.social, 30);
    }

    #[test]
    fn artist_definition_to_artist() {
        let def = ArtistDefinition {
            id: ArtistId(1),
            name: "Test".to_string(),
            starting_age: 20,
            base_attributes: BaseAttributes::default(),
            personality: PersonalitySpectrums::default(),
            traits: InnerTraits::default(),
            image: ImageTags::default(),
        };
        let artist = def.into_artist();
        assert_eq!(artist.id, ArtistId(1));
        assert_eq!(artist.age, 20);
        assert_eq!(artist.stats, AuxiliaryStats::default());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p stardom-core data_loader`
Expected: FAIL

- [ ] **Step 3: Implement ArtistDefinition and loader**

```rust
use serde::{Deserialize, Serialize};

use crate::artist::Artist;
use crate::attribute::{BaseAttributes, InnerTraits};
use crate::persona::{ImageTags, PersonalitySpectrums};
use crate::stats::AuxiliaryStats;
use crate::types::{Activity, ArtistId};

/// Data definition for an artist, loaded from RON files.
/// This is the "template" — it creates an Artist instance at game start.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistDefinition {
    pub id: ArtistId,
    pub name: String,
    pub starting_age: u32,
    pub base_attributes: BaseAttributes,
    pub personality: PersonalitySpectrums,
    pub traits: InnerTraits,
    pub image: ImageTags,
}

impl ArtistDefinition {
    pub fn into_artist(self) -> Artist {
        Artist {
            id: self.id,
            name: self.name,
            age: self.starting_age,
            base_attributes: self.base_attributes,
            skills: Default::default(),
            traits: self.traits,
            personality: self.personality,
            image: self.image,
            stats: AuxiliaryStats::default(),
            current_activity: Activity::Idle,
            inactive_weeks: 0,
        }
    }
}

pub fn load_artist_definition(ron_str: &str) -> Result<ArtistDefinition, ron::error::SpannedError> {
    ron::from_str(ron_str)
}
```

- [ ] **Step 4: Create sample artist RON file**

`data/artists/sample_artist.ron`:
```ron
ArtistDefinition(
    id: ArtistId(1),
    name: "Luna Star",
    starting_age: 18,
    base_attributes: BaseAttributes(
        stamina: 60,
        intellect: 55,
        empathy: 70,
        charm: 80,
    ),
    personality: PersonalitySpectrums(
        social: 30,
        thinking: -20,
        action: 10,
        stance: -40,
    ),
    traits: InnerTraits(
        confidence: 55,
        rebellion: 25,
    ),
    image: ImageTags(
        pure: 60,
        sexy: 20,
        cool: 40,
        intellectual: 30,
        funny: 10,
        mysterious: 50,
    ),
)
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p stardom-core data_loader`
Expected: 2 tests PASS

- [ ] **Step 6: Commit**

```bash
git add crates/stardom-core/src/data_loader.rs data/
git commit -m "feat(core): add ArtistDefinition RON loader with sample data"
```

---

## Task 11: GameState & GameCommand (Top-Level Game Loop)

**Files:**
- Create: `crates/stardom-core/src/game.rs`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::BaseAttributes;
    use crate::config::Settings;
    use crate::types::{ArtistId, Money};

    fn make_test_game() -> GameState {
        GameState::new(Settings::default())
    }

    #[test]
    fn new_game_state() {
        let game = make_test_game();
        assert_eq!(game.calendar.year, 1);
        assert_eq!(game.calendar.week, 1);
        assert_eq!(game.company.balance, Money(1_000_000));
        assert!(game.artists.is_empty());
    }

    #[test]
    fn advance_week_updates_calendar() {
        let mut game = make_test_game();
        game.process_command(GameCommand::AdvanceWeek);
        assert_eq!(game.calendar.week, 2);
        assert_eq!(game.calendar.total_weeks_elapsed, 1);
    }

    #[test]
    fn advance_week_decays_popularity() {
        let mut game = make_test_game();
        let mut artist = Artist::new(
            ArtistId(1),
            "Test".to_string(),
            20,
            BaseAttributes::default(),
        );
        artist.stats.popularity = 50;
        game.artists.push(artist);

        game.process_command(GameCommand::AdvanceWeek);
        // Base decay -2, inactive 1 week penalty -2 = -4
        assert_eq!(game.artists[0].stats.popularity, 46);
    }

    #[test]
    fn game_phase_transitions() {
        let mut game = make_test_game();
        assert_eq!(game.phase, GamePhase::MainGame);

        // Advance past goal period (3 years = 156 weeks)
        for _ in 0..156 {
            game.process_command(GameCommand::AdvanceWeek);
        }
        assert_eq!(game.phase, GamePhase::PostEnding);
    }

    #[test]
    fn artists_age_on_year_rollover() {
        let mut game = make_test_game();
        let artist = Artist::new(ArtistId(1), "Test".to_string(), 20, BaseAttributes::default());
        game.artists.push(artist);

        // Advance 52 weeks (1 year)
        for _ in 0..52 {
            game.process_command(GameCommand::AdvanceWeek);
        }
        assert_eq!(game.artists[0].age, 21);
    }

    #[test]
    fn bankruptcy_ends_game() {
        let settings = Settings { starting_balance: -1, ..Settings::default() };
        let mut game = GameState::new(settings);
        // Force 4 weeks of negative balance with no artists (no pending income)
        for _ in 0..4 {
            game.process_command(GameCommand::AdvanceWeek);
        }
        assert_eq!(game.phase, GamePhase::GameOver);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p stardom-core game`
Expected: FAIL

- [ ] **Step 3: Implement GameState and GameCommand**

```rust
use serde::{Deserialize, Serialize};

use crate::artist::Artist;
use crate::calendar::Calendar;
use crate::company::CompanyState;
use crate::config::Settings;
use crate::types::{Activity, ArtistId, Money};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    MainGame,
    PostEnding,
    GameOver,
}

#[derive(Debug, Clone)]
pub enum GameCommand {
    AdvanceWeek,
    // Future commands will be added in later phases:
    // AssignActivity { artist_id: ArtistId, activity: Activity },
    // SignArtist { artist_id: ArtistId, commission: f32 },
    // etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub calendar: Calendar,
    pub company: CompanyState,
    pub artists: Vec<Artist>,
    pub phase: GamePhase,
    pub settings: Settings,
}

impl GameState {
    pub fn new(settings: Settings) -> Self {
        Self {
            calendar: Calendar::new(settings.goal_years),
            company: CompanyState::new(Money(settings.starting_balance), settings.max_artists),
            artists: Vec::new(),
            phase: GamePhase::MainGame,
            settings,
        }
    }

    pub fn process_command(&mut self, command: GameCommand) {
        if self.phase == GamePhase::GameOver {
            return;
        }

        match command {
            GameCommand::AdvanceWeek => self.advance_week(),
        }
    }

    fn advance_week(&mut self) {
        let was_last_week_of_year = self.calendar.week == 52;
        self.calendar.advance_week();

        // Age artists on year rollover
        if was_last_week_of_year {
            for artist in &mut self.artists {
                artist.age += 1;
            }
        }

        // Update each artist's popularity decay
        for artist in &mut self.artists {
            let active = artist.current_activity.is_public();
            artist.inactive_weeks = if active { 0 } else { artist.inactive_weeks + 1 };
            artist.stats.apply_weekly_popularity_decay(active, artist.inactive_weeks);
            // Reset activity to Idle for next week's assignment
            artist.current_activity = Activity::Idle;
        }

        // Update bankruptcy counter
        let has_pending_income = false; // TODO: check pending gig income
        self.company.update_bankruptcy_counter(has_pending_income);

        // Check phase transitions
        if self.company.is_bankrupt() {
            self.phase = GamePhase::GameOver;
        } else if self.phase == GamePhase::MainGame && self.calendar.is_goal_period_over() {
            self.phase = GamePhase::PostEnding;
        }
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p stardom-core game`
Expected: 5 tests PASS

- [ ] **Step 5: Run full test suite**

Run: `cargo test --workspace`
Expected: all tests PASS (approximately 50 tests across all modules)

- [ ] **Step 6: Commit**

```bash
git add crates/stardom-core/src/game.rs
git commit -m "feat(core): add GameState with week advancement, phase transitions, and bankruptcy check"
```

---

## Task 12: Wire Up lib.rs Exports & Final Integration Test

**Files:**
- Modify: `crates/stardom-core/src/lib.rs`
- Test: integration test at end of `lib.rs` or as a separate test

- [ ] **Step 1: Write integration test**

Add to `crates/stardom-core/src/lib.rs`:
```rust
#[cfg(test)]
mod integration_tests {
    use crate::config::Settings;
    use crate::data_loader::load_artist_definition;
    use crate::game::{GameCommand, GamePhase, GameState};

    #[test]
    fn full_game_loop_smoke_test() {
        // Load settings
        let settings = Settings::default();

        // Create game
        let mut game = GameState::new(settings);
        assert_eq!(game.phase, GamePhase::MainGame);

        // Load and sign an artist
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

        assert_eq!(game.artists.len(), 1);
        assert_eq!(game.artists[0].name, "Luna Star");

        // Advance a full year
        for _ in 0..52 {
            game.process_command(GameCommand::AdvanceWeek);
        }
        assert_eq!(game.calendar.year, 2);
        assert_eq!(game.phase, GamePhase::MainGame);

        // Advance to end of goal period
        for _ in 0..(52 * 2) {
            game.process_command(GameCommand::AdvanceWeek);
        }
        assert_eq!(game.calendar.year, 4);
        assert_eq!(game.phase, GamePhase::PostEnding);

        // Popularity should have decayed to 0 (no activity)
        assert_eq!(game.artists[0].stats.popularity, 0);
    }
}
```

- [ ] **Step 2: Run integration test**

Run: `cargo test -p stardom-core integration_tests`
Expected: 1 test PASS

- [ ] **Step 3: Run full workspace test suite**

Run: `cargo test --workspace`
Expected: all tests PASS

- [ ] **Step 4: Commit**

```bash
git add crates/stardom-core/src/lib.rs
git commit -m "test(core): add full game loop integration smoke test"
```

---

## Task 13: Final Cleanup & Push

- [ ] **Step 1: Run clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: no warnings

- [ ] **Step 2: Run fmt check**

Run: `cargo fmt --check --all`
Expected: no formatting issues (run `cargo fmt --all` if needed)

- [ ] **Step 3: Verify Bevy app launches**

Run: `cargo run -p redo-stardom`
Expected: a window titled "REDÓ Stardom" (800x600) opens. Close it manually.

- [ ] **Step 4: Final commit if any cleanup was needed**

```bash
git add -A
git commit -m "chore: clippy and fmt cleanup"
```

- [ ] **Step 5: Push**

```bash
git push
```

---

## Phase 1 Completion Checklist

After all tasks, verify:

- [ ] `cargo test --workspace` — all tests pass
- [ ] `cargo clippy --workspace -- -D warnings` — no warnings
- [ ] `cargo fmt --check --all` — clean
- [ ] `cargo run -p redo-stardom` — window opens
- [ ] All data model types serialize/deserialize with RON
- [ ] Settings load from TOML
- [ ] Calendar advances correctly through 3+ years
- [ ] Popularity decays weekly
- [ ] Bankruptcy triggers after 4 negative weeks
- [ ] Game phase transitions (MainGame → PostEnding → GameOver)

---

## What Phase 2 Will Cover

> Not part of this plan. Listed for context only.

- Training system (with efficiency formulas from Appendix A.1/A.2)
- Part-time job system
- Activity scheduling (assign weekly activities to artists)
- Gig definitions and availability rotation
- Financial income/expense cycle
