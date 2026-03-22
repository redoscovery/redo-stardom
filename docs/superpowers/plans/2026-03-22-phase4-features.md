# Phase 4: Features — Outfit, Recruitment, Mini-Game, Narrative, Save/Load

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the remaining v1 game systems: costume/outfit system with image tag modifiers, artist recruitment with contract negotiation, mini-game auto-resolve framework, narrative scripting engine with branching dialogue, and save/load persistence.

**Architecture:** Each system gets its own module. The outfit system adds an `equipped_outfit` field to Artist. Recruitment extends `data_loader.rs` with location/schedule data and adds a `SignArtist` command. The mini-game module provides auto-resolve logic (actual mini-games are Bevy-side). The narrative engine is a simple condition-based script runner using RON. Save/load serializes the full `GameState` to RON files.

**Tech Stack:** Rust 1.94, serde + ron, rand, cargo test

**Spec reference:** `docs/superpowers/specs/2026-03-22-redo-stardom-design.md` (Sections 4.7, 5.3, 5.4, 6, 7, A.8, A.9)

---

## File Structure

```
crates/stardom-core/src/
├── outfit.rs        # OutfitDef, equip logic, temporary image modifier
├── recruitment.rs   # RecruitmentState, scouting, contract negotiation
├── minigame.rs      # MiniGameDef, auto-resolve formula
├── narrative.rs     # ScriptDef, Condition, DialogueNode, script runner
├── save.rs          # save/load GameState to/from RON files
├── artist.rs        # (modify) add equipped_outfit, commission_rate fields
├── data_loader.rs   # (modify) extend ArtistDefinition with location/schedule
├── game.rs          # (modify) add new commands, recruitment/narrative hooks
├── types.rs         # (modify) add NpcId, ScriptId
├── lib.rs           # (modify) add new pub mod declarations
data/
├── outfits/
│   └── default_outfits.ron
├── scripts/
│   └── sample_script.ron
└── artists/
    └── sample_artist.ron     # (modify) add recruitment fields
```

---

## Task 1: Outfit System

**Files:**
- Create: `crates/stardom-core/src/outfit.rs`
- Create: `data/outfits/default_outfits.ron`
- Modify: `crates/stardom-core/src/artist.rs` — add `equipped_outfit: Option<OutfitId>`
- Modify: `crates/stardom-core/src/types.rs` — add OutfitId if not present (already exists in id_newtype!)
- Modify: `crates/stardom-core/src/lib.rs`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::persona::ImageTags;

    fn sample_outfit() -> OutfitDef {
        OutfitDef {
            id: OutfitId(1),
            name: "Elegant Dress".into(),
            cost: Money(50_000),
            image_modifiers: vec![(ImageTag::Pure, 15), (ImageTag::Intellectual, 10)],
            trait_modifiers: vec![],
        }
    }

    #[test]
    fn apply_outfit_modifiers() {
        let outfit = sample_outfit();
        let base = ImageTags { pure: 40, intellectual: 20, ..Default::default() };
        let modified = outfit.apply_to_image(&base);
        assert_eq!(modified.pure, 55);       // 40 + 15
        assert_eq!(modified.intellectual, 30); // 20 + 10
        assert_eq!(modified.sexy, 0);         // unchanged
    }

    #[test]
    fn outfit_modifiers_clamp_to_max() {
        let outfit = OutfitDef {
            id: OutfitId(2),
            name: "Wild Outfit".into(),
            cost: Money(80_000),
            image_modifiers: vec![(ImageTag::Sexy, 50)],
            trait_modifiers: vec![],
        };
        let base = ImageTags { sexy: 80, ..Default::default() };
        let modified = outfit.apply_to_image(&base);
        assert_eq!(modified.sexy, 100); // clamped
    }

    #[test]
    fn serialization_roundtrip() {
        let outfit = sample_outfit();
        let s = ron::to_string(&outfit).unwrap();
        let d: OutfitDef = ron::from_str(&s).unwrap();
        assert_eq!(d.name, "Elegant Dress");
        assert_eq!(d.image_modifiers.len(), 2);
    }
}
```

- [ ] **Step 2: Implement outfit module**

```rust
use serde::{Deserialize, Serialize};
use crate::persona::{ImageTag, ImageTags, IMAGE_TAG_MAX, IMAGE_TAG_MIN};
use crate::types::{Money, OutfitId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutfitDef {
    pub id: OutfitId,
    pub name: String,
    pub cost: Money,
    pub image_modifiers: Vec<(ImageTag, i32)>,
    pub trait_modifiers: Vec<(TraitModifier, i32)>,  // for future use (e.g., Rebellion +5)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraitModifier {
    Confidence,
    Rebellion,
}

impl OutfitDef {
    /// Apply outfit modifiers on top of base image tags, returning a new ImageTags.
    /// Values are clamped to 0-100.
    pub fn apply_to_image(&self, base: &ImageTags) -> ImageTags {
        let mut result = *base;
        for (tag, modifier) in &self.image_modifiers {
            let current = tag.value_from(&result);
            let new_val = (current + modifier).clamp(IMAGE_TAG_MIN, IMAGE_TAG_MAX);
            set_image_tag(&mut result, *tag, new_val);
        }
        result
    }
}

fn set_image_tag(tags: &mut ImageTags, tag: ImageTag, value: i32) {
    match tag {
        ImageTag::Pure => tags.pure = value,
        ImageTag::Sexy => tags.sexy = value,
        ImageTag::Cool => tags.cool = value,
        ImageTag::Intellectual => tags.intellectual = value,
        ImageTag::Funny => tags.funny = value,
        ImageTag::Mysterious => tags.mysterious = value,
    }
}
```

- [ ] **Step 3: Add `equipped_outfit: Option<OutfitId>` to Artist** (default None, add to new() and into_artist)

- [ ] **Step 4: Create `data/outfits/default_outfits.ron`** with 4-5 sample outfits

- [ ] **Step 5: Add `PurchaseOutfit` and `EquipOutfit` commands to game.rs**

```rust
// In GameCommand:
PurchaseOutfit { outfit_id: OutfitId },
EquipOutfit { artist_index: usize, outfit_id: OutfitId },

// GameState gains:
pub outfit_catalog: Vec<OutfitDef>,
pub owned_outfits: Vec<OutfitId>,
```

- [ ] **Step 6: Run tests, commit**

```bash
git commit -m "feat(core): add outfit system with image tag modifiers and equip logic"
```

---

## Task 2: Artist Recruitment System

**Files:**
- Create: `crates/stardom-core/src/recruitment.rs`
- Modify: `crates/stardom-core/src/data_loader.rs` — extend ArtistDefinition
- Modify: `crates/stardom-core/src/artist.rs` — add `commission_rate: f32`
- Modify: `crates/stardom-core/src/types.rs` — ensure types are available
- Modify: `crates/stardom-core/src/game.rs` — add SignArtist command

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_prospect() -> ArtistProspect {
        ArtistProspect {
            definition: ArtistDefinition { /* ... */ },
            location: "Cafe".into(),
            available_day: 3, // Wednesday (1=Mon, 7=Sun)
            base_commission: 0.30,
            failed_attempts: 0,
            locked_until_week: 0,
        }
    }

    #[test]
    fn prospect_available_on_correct_day() {
        let prospect = sample_prospect();
        assert!(prospect.is_available(3, 10)); // day 3, week 10
        assert!(!prospect.is_available(1, 10)); // wrong day
    }

    #[test]
    fn prospect_locked_after_two_failures() {
        let mut prospect = sample_prospect();
        prospect.failed_attempts = 1;
        assert!(!prospect.is_locked(10));
        prospect.failed_attempts = 2;
        prospect.locked_until_week = 36; // locked for 26 weeks
        assert!(prospect.is_locked(10));
        assert!(!prospect.is_locked(36));
    }

    #[test]
    fn negotiate_commission_adjustment() {
        let prospect = sample_prospect();
        // Good choice: -5% commission
        assert!((negotiate_commission(prospect.base_commission, -5) - 0.25).abs() < 0.001);
        // Bad choice: +10% commission
        assert!((negotiate_commission(prospect.base_commission, 10) - 0.40).abs() < 0.001);
    }

    #[test]
    fn commission_clamped_to_range() {
        assert!((negotiate_commission(0.10, -10) - 0.15).abs() < 0.001); // floor 15%
        assert!((negotiate_commission(0.45, 10) - 0.50).abs() < 0.001); // cap 50%
    }
}
```

- [ ] **Step 2: Implement recruitment module**

```rust
use serde::{Deserialize, Serialize};
use crate::data_loader::ArtistDefinition;

const MIN_COMMISSION: f32 = 0.15;
const MAX_COMMISSION: f32 = 0.50;
const LOCKOUT_WEEKS: u32 = 26;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistProspect {
    pub definition: ArtistDefinition,
    pub location: String,
    pub available_day: u32, // 1=Mon .. 7=Sun
    pub base_commission: f32,
    pub failed_attempts: u32,
    pub locked_until_week: u32,
}

impl ArtistProspect {
    pub fn is_available(&self, day_of_week: u32, current_week: u32) -> bool {
        self.available_day == day_of_week && !self.is_locked(current_week)
    }

    pub fn is_locked(&self, current_week: u32) -> bool {
        self.failed_attempts >= 2 && current_week < self.locked_until_week
    }

    pub fn record_failure(&mut self, current_week: u32) {
        self.failed_attempts += 1;
        if self.failed_attempts >= 2 {
            self.locked_until_week = current_week + LOCKOUT_WEEKS;
        }
    }
}

pub fn negotiate_commission(base: f32, adjustment_pct: i32) -> f32 {
    let adjusted = base + (adjustment_pct as f32 / 100.0);
    adjusted.clamp(MIN_COMMISSION, MAX_COMMISSION)
}
```

- [ ] **Step 3: Add `commission_rate: f32` to Artist** (default 0.30, initialized in new())

- [ ] **Step 4: Add `SignArtist` command to game.rs**

```rust
// GameCommand:
SignArtist { prospect_index: usize, commission_adjustment: i32 },

// GameState gains:
pub prospects: Vec<ArtistProspect>,
```

SignArtist: check company.max_artists, negotiate commission, create artist from prospect, push to artists vec.

- [ ] **Step 5: Run tests, commit**

```bash
git commit -m "feat(core): add artist recruitment with scouting and contract negotiation"
```

---

## Task 3: Mini-Game Auto-Resolve Framework

**Files:**
- Create: `crates/stardom-core/src/minigame.rs`
- Modify: `crates/stardom-core/src/lib.rs`

The actual mini-games run in Bevy. This module only provides the auto-resolve formula and type definitions.

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_resolve_passes_with_high_skill() {
        // skill=5000, threshold=3000
        // score = 5000*0.7 + roll_component = 3500 + something
        // Even with roll=0: 3500 >= 3000 → pass
        let result = auto_resolve(5000, 3000, 0.0);
        assert!(result.passed);
        assert_eq!(result.rating, MiniGameRating::Standard);
    }

    #[test]
    fn auto_resolve_excellent_at_120pct() {
        // skill=5000, threshold=3000
        // Need score >= 3600 (3000 * 1.2) for excellent
        // score = 5000*0.7 + 5000*0.3*1.0 = 3500 + 1500 = 5000 → excellent
        let result = auto_resolve(5000, 3000, 1.0);
        assert!(result.passed);
        assert_eq!(result.rating, MiniGameRating::Excellent);
    }

    #[test]
    fn auto_resolve_fails_with_low_skill() {
        // skill=1000, threshold=3000
        // max score = 1000*0.7 + 1000*0.3 = 1000 → < 3000, fails
        let result = auto_resolve(1000, 3000, 1.0);
        assert!(!result.passed);
        assert_eq!(result.rating, MiniGameRating::Failed);
    }

    #[test]
    fn minigame_def_serialization() {
        let def = MiniGameDef {
            id: MiniGameId(1),
            name: "Rhythm Challenge".into(),
            category: MiniGameCategory::Rhythm,
            difficulty_threshold: 3000,
            relevant_skill: SkillTarget::Vocal,
        };
        let s = ron::to_string(&def).unwrap();
        let d: MiniGameDef = ron::from_str(&s).unwrap();
        assert_eq!(d.name, "Rhythm Challenge");
    }
}
```

- [ ] **Step 2: Implement minigame module**

```rust
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

/// Auto-resolve formula from spec 5.4:
/// score = skill * 0.7 + random(0, skill * 0.3)
/// `roll` is a normalized random value 0.0..1.0
pub fn auto_resolve(skill: i32, threshold: i32, roll: f64) -> MiniGameResult {
    let base = skill as f64 * 0.7;
    let random_part = skill as f64 * 0.3 * roll.clamp(0.0, 1.0);
    let score = (base + random_part) as i32;

    let passed = score >= threshold;
    let excellent_threshold = (threshold as f64 * 1.2) as i32;
    let rating = if !passed {
        MiniGameRating::Failed
    } else if score >= excellent_threshold {
        MiniGameRating::Excellent
    } else {
        MiniGameRating::Standard
    };

    MiniGameResult { passed, rating, score }
}
```

- [ ] **Step 3: Run tests, commit**

```bash
git commit -m "feat(core): add mini-game auto-resolve framework"
```

---

## Task 4: Narrative Scripting Engine

**Files:**
- Create: `crates/stardom-core/src/narrative.rs`
- Create: `data/scripts/sample_script.ron`
- Modify: `crates/stardom-core/src/types.rs` — add ScriptId, NpcId
- Modify: `crates/stardom-core/src/lib.rs`

This is the event/dialogue framework. Scripts are data-driven trees of dialogue nodes with conditions.

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_script() -> ScriptDef {
        ScriptDef {
            id: ScriptId(1),
            name: "Award Ceremony".into(),
            trigger: ScriptTrigger::MonthReached(12),
            nodes: vec![
                DialogueNode {
                    id: 0,
                    speaker: "Host".into(),
                    text: "The winner is...".into(),
                    choices: vec![],
                    next: Some(1),
                    condition: None,
                },
                DialogueNode {
                    id: 1,
                    speaker: "Host".into(),
                    text: "Congratulations!".into(),
                    choices: vec![
                        DialogueChoice { label: "Thank you!".into(), next_node: Some(2), effects: vec![] },
                        DialogueChoice { label: "I deserve this.".into(), next_node: Some(2), effects: vec![ScriptEffect::ChangeReputation(5)] },
                    ],
                    next: None,
                    condition: None,
                },
                DialogueNode {
                    id: 2,
                    speaker: "".into(),
                    text: "The ceremony ends.".into(),
                    choices: vec![],
                    next: None,
                    condition: None,
                },
            ],
        }
    }

    #[test]
    fn script_starts_at_first_node() {
        let script = sample_script();
        let runner = ScriptRunner::new(&script);
        assert_eq!(runner.current_node_id(), 0);
    }

    #[test]
    fn advance_follows_next() {
        let script = sample_script();
        let mut runner = ScriptRunner::new(&script);
        runner.advance(None);
        assert_eq!(runner.current_node_id(), 1);
    }

    #[test]
    fn advance_with_choice() {
        let script = sample_script();
        let mut runner = ScriptRunner::new(&script);
        runner.advance(None); // 0 → 1
        let effects = runner.advance(Some(1)); // choice 1: "I deserve this" → node 2
        assert_eq!(runner.current_node_id(), 2);
        assert_eq!(effects.len(), 1);
        assert!(matches!(effects[0], ScriptEffect::ChangeReputation(5)));
    }

    #[test]
    fn runner_is_finished_at_terminal_node() {
        let script = sample_script();
        let mut runner = ScriptRunner::new(&script);
        runner.advance(None); // 0 → 1
        runner.advance(Some(0)); // 1 → 2
        assert!(!runner.is_finished());
        runner.advance(None); // 2 → None (terminal)
        assert!(runner.is_finished());
    }

    #[test]
    fn trigger_matches_month() {
        let script = sample_script();
        assert!(script.trigger.matches_month(12));
        assert!(!script.trigger.matches_month(6));
    }

    #[test]
    fn serialization_roundtrip() {
        let script = sample_script();
        let s = ron::to_string(&script).unwrap();
        let d: ScriptDef = ron::from_str(&s).unwrap();
        assert_eq!(d.name, "Award Ceremony");
        assert_eq!(d.nodes.len(), 3);
    }
}
```

- [ ] **Step 2: Implement narrative module**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScriptId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptTrigger {
    MonthReached(u32),
    WeekReached(u32),
    StatThreshold { stat: String, value: i32 },
    Manual, // triggered by game logic directly
}

impl ScriptTrigger {
    pub fn matches_month(&self, month: u32) -> bool {
        matches!(self, ScriptTrigger::MonthReached(m) if *m == month)
    }

    pub fn matches_week(&self, week: u32) -> bool {
        matches!(self, ScriptTrigger::WeekReached(w) if *w == week)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptEffect {
    ChangeReputation(i32),
    ChangePopularity(i32),
    ChangeStress(i32),
    ChangeRecognition(i64),
    AddMoney(i64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueChoice {
    pub label: String,
    pub next_node: Option<u32>,
    pub effects: Vec<ScriptEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    pub id: u32,
    pub speaker: String,
    pub text: String,
    pub choices: Vec<DialogueChoice>,
    pub next: Option<u32>, // auto-advance if no choices
    pub condition: Option<String>, // placeholder for conditions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptDef {
    pub id: ScriptId,
    pub name: String,
    pub trigger: ScriptTrigger,
    pub nodes: Vec<DialogueNode>,
}

impl ScriptDef {
    pub fn get_node(&self, id: u32) -> Option<&DialogueNode> {
        self.nodes.iter().find(|n| n.id == id)
    }
}

/// Runs through a script's dialogue tree.
pub struct ScriptRunner<'a> {
    script: &'a ScriptDef,
    current: Option<u32>,
}

impl<'a> ScriptRunner<'a> {
    pub fn new(script: &'a ScriptDef) -> Self {
        let first = script.nodes.first().map(|n| n.id);
        Self { script, current: first }
    }

    pub fn current_node_id(&self) -> u32 {
        self.current.unwrap_or(u32::MAX)
    }

    pub fn current_node(&self) -> Option<&DialogueNode> {
        self.current.and_then(|id| self.script.get_node(id))
    }

    pub fn is_finished(&self) -> bool {
        self.current.is_none()
    }

    /// Advance to next node. If choice_index is Some, follow that choice.
    /// Returns any effects from the chosen path.
    pub fn advance(&mut self, choice_index: Option<usize>) -> Vec<ScriptEffect> {
        let Some(node) = self.current_node() else {
            self.current = None;
            return vec![];
        };

        let (next_id, effects) = if let Some(idx) = choice_index {
            if let Some(choice) = node.choices.get(idx) {
                (choice.next_node, choice.effects.clone())
            } else {
                (node.next, vec![])
            }
        } else {
            (node.next, vec![])
        };

        self.current = next_id;
        effects
    }
}
```

- [ ] **Step 3: Create `data/scripts/sample_script.ron`**

- [ ] **Step 4: Run tests, commit**

```bash
git commit -m "feat(core): add narrative scripting engine with branching dialogue"
```

---

## Task 5: Save/Load System

**Files:**
- Create: `crates/stardom-core/src/save.rs`
- Modify: `crates/stardom-core/src/lib.rs`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;
    use crate::game::GameState;

    #[test]
    fn serialize_and_deserialize_game_state() {
        let game = GameState::new(Settings::default());
        let data = save_to_string(&game).unwrap();
        let loaded: GameState = load_from_string(&data).unwrap();
        assert_eq!(loaded.calendar.year, game.calendar.year);
        assert_eq!(loaded.company.balance, game.company.balance);
        assert_eq!(loaded.phase, game.phase);
    }

    #[test]
    fn roundtrip_with_artists() {
        let mut game = GameState::new(Settings::default());
        let artist = crate::artist::Artist::new(
            crate::types::ArtistId(1),
            "Test".into(),
            20,
            crate::attribute::BaseAttributes::default(),
        );
        game.artists.push(artist);
        let data = save_to_string(&game).unwrap();
        let loaded: GameState = load_from_string(&data).unwrap();
        assert_eq!(loaded.artists.len(), 1);
        assert_eq!(loaded.artists[0].name, "Test");
    }

    #[test]
    fn save_to_file_and_load(tmp: /* use std::env::temp_dir */) {
        let game = GameState::new(Settings::default());
        let path = std::env::temp_dir().join("test_save.ron");
        save_to_file(&game, &path).unwrap();
        let loaded: GameState = load_from_file(&path).unwrap();
        assert_eq!(loaded.calendar.week, 1);
        std::fs::remove_file(path).ok();
    }
}
```

- [ ] **Step 2: Implement save module**

```rust
use std::path::Path;
use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug)]
pub enum SaveError {
    Serialize(ron::Error),
    Deserialize(ron::error::SpannedError),
    Io(std::io::Error),
}

impl From<ron::Error> for SaveError {
    fn from(e: ron::Error) -> Self { Self::Serialize(e) }
}
impl From<ron::error::SpannedError> for SaveError {
    fn from(e: ron::error::SpannedError) -> Self { Self::Deserialize(e) }
}
impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self { Self::Io(e) }
}

pub fn save_to_string<T: Serialize>(state: &T) -> Result<String, SaveError> {
    Ok(ron::ser::to_string_pretty(state, ron::ser::PrettyConfig::default())?)
}

pub fn load_from_string<T: DeserializeOwned>(data: &str) -> Result<T, SaveError> {
    Ok(ron::from_str(data)?)
}

pub fn save_to_file<T: Serialize>(state: &T, path: &Path) -> Result<(), SaveError> {
    let data = save_to_string(state)?;
    std::fs::write(path, data)?;
    Ok(())
}

pub fn load_from_file<T: DeserializeOwned>(path: &Path) -> Result<T, SaveError> {
    let data = std::fs::read_to_string(path)?;
    load_from_string(&data)
}
```

- [ ] **Step 3: Run tests, commit**

```bash
git commit -m "feat(core): add save/load system with RON serialization"
```

---

## Task 6: Game Loop Integration & Final Cleanup

**Files:**
- Modify: `crates/stardom-core/src/game.rs`
- Modify: `crates/stardom-core/src/lib.rs` (integration test)

- [ ] **Step 1: Add PurchaseOutfit/EquipOutfit/SignArtist commands to game.rs**

```rust
// New GameCommand variants:
PurchaseOutfit { outfit_id: OutfitId },
EquipOutfit { artist_index: usize, outfit_id: OutfitId },
SignArtist { prospect_index: usize, commission_adjustment: i32 },
```

New GameState fields (all `#[serde(default)]`):
```rust
pub outfit_catalog: Vec<OutfitDef>,
pub owned_outfits: Vec<OutfitId>,
pub prospects: Vec<ArtistProspect>,
pub script_catalog: Vec<ScriptDef>,
```

- [ ] **Step 2: Implement command handlers**

PurchaseOutfit: check if affordable, spend money, add to owned_outfits.
EquipOutfit: check if owned, set artist.equipped_outfit.
SignArtist: check roster size < max_artists, negotiate commission, create artist, remove from prospects.

- [ ] **Step 3: Update integration test**

Test that covers: create game → purchase outfit → equip outfit → sign artist → save → load → verify state preserved.

- [ ] **Step 4: Run clippy and fmt**

```bash
cargo fmt --all && cargo clippy --workspace -- -D warnings
```

- [ ] **Step 5: Run full test suite**

```bash
cargo test --workspace
```

- [ ] **Step 6: Commit**

```bash
git commit -m "feat(core): integrate outfit, recruitment, and narrative into game loop"
```

---

## Phase 4 Completion Checklist

- [ ] `cargo test --workspace` — all tests pass
- [ ] `cargo clippy --workspace -- -D warnings` — clean
- [ ] `cargo fmt --check --all` — clean
- [ ] Outfit: purchase, equip, image tag modifiers applied
- [ ] Recruitment: prospect scouting, commission negotiation, signing, lockout
- [ ] Mini-game: auto-resolve formula, Standard/Excellent/Failed ratings
- [ ] Narrative: ScriptDef, branching DialogueNodes, ScriptRunner, effects
- [ ] Save/Load: GameState to/from RON string and file

---

## What Phase 5 Will Cover

> Not part of this plan. Listed for context only.

- Bevy presentation layer: main menu, game UI, artist management screens
- Pixel art rendering pipeline
- Input handling and UI interaction
- Audio system hooks
