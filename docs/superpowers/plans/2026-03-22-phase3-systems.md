# Phase 3: Game Systems — Gig Rotation, Awards, PR Crisis, Office Upgrades

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the four remaining v1 game systems: bi-weekly gig rotation pool, award ceremonies with nomination/judging, PR crisis random events with player choices, and office upgrade progression — all data-driven and wired into the game loop.

**Architecture:** Each system gets its own module with data definitions (RON) and logic. The game loop (`game.rs`) is extended with new commands and weekly/monthly event hooks. `rand` crate (already in dependencies) is used for random event rolls.

**Tech Stack:** Rust 1.94, serde + ron, rand, cargo test

**Spec reference:** `docs/superpowers/specs/2026-03-22-redo-stardom-design.md` (Sections 4.4, 4.5, 5.1, 5.2)

---

## File Structure

```
crates/stardom-core/src/
├── gig_pool.rs      # bi-weekly gig rotation pool generation
├── award.rs         # AwardDef, nomination, judging, ceremony timing
├── crisis.rs        # CrisisDef, CrisisChoice, random trigger, resolution
├── office.rs        # OfficeUpgradeDef, upgrade logic, bonuses
├── game.rs          # (modify) add new commands, event hooks in advance_week
├── company.rs       # (modify) add upgrade/downgrade methods
├── config.rs        # (modify) add crisis_chance_base setting
├── lib.rs           # (modify) add new pub mod declarations
data/
├── gigs/
│   └── default_gigs.ron      # (already exists, may add more entries)
├── awards/
│   └── default_awards.ron    # 3 award definitions
├── crises/
│   └── default_crises.ron    # sample crisis scenarios
└── offices/
    └── default_offices.ron   # office tier definitions with costs and bonuses
```

---

## Task 1: Gig Pool — Bi-Weekly Rotation

**Files:**
- Create: `crates/stardom-core/src/gig_pool.rs`
- Modify: `crates/stardom-core/src/lib.rs`
- Modify: `crates/stardom-core/src/game.rs`

The gig pool generates a set of available gigs each bi-weekly rotation from the full gig catalog. Players pick from the pool. The pool refreshes every 2 weeks.

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gig::{GigCategory, GigDef};
    use crate::stats::RecognitionTier;
    use crate::types::GigId;

    fn make_catalog() -> Vec<GigDef> {
        vec![
            GigDef {
                id: GigId(1), name: "Single A".into(), category: GigCategory::Music,
                duration_weeks: 1, required_recognition_tier: RecognitionTier::Unknown,
                skill_weights: vec![], base_pay: 10_000, recognition_reward: 10,
                reputation_reward: 1, stress_cost: 5,
                ideal_image_tags: vec![], conflicting_image_tags: vec![],
                personality_preference: None, skill_gains: vec![],
            },
            GigDef {
                id: GigId(2), name: "Drama B".into(), category: GigCategory::FilmTv,
                duration_weeks: 3, required_recognition_tier: RecognitionTier::Rising,
                skill_weights: vec![], base_pay: 80_000, recognition_reward: 50,
                reputation_reward: 3, stress_cost: 12,
                ideal_image_tags: vec![], conflicting_image_tags: vec![],
                personality_preference: None, skill_gains: vec![],
            },
            GigDef {
                id: GigId(3), name: "Ad C".into(), category: GigCategory::Endorsement,
                duration_weeks: 1, required_recognition_tier: RecognitionTier::Unknown,
                skill_weights: vec![], base_pay: 15_000, recognition_reward: 5,
                reputation_reward: 0, stress_cost: 3,
                ideal_image_tags: vec![], conflicting_image_tags: vec![],
                personality_preference: None, skill_gains: vec![],
            },
        ]
    }

    #[test]
    fn generate_pool_filters_by_rotation() {
        let catalog = make_catalog();
        // rotation_a: Music + FilmTv, rotation_b: Modeling + Variety + Endorsement + Creative
        let pool_a = generate_pool(&catalog, true);
        let pool_b = generate_pool(&catalog, false);
        assert!(pool_a.iter().all(|g| matches!(g.category, GigCategory::Music | GigCategory::FilmTv)));
        assert!(pool_b.iter().all(|g| !matches!(g.category, GigCategory::Music | GigCategory::FilmTv)));
    }

    #[test]
    fn filter_available_for_artist() {
        let catalog = make_catalog();
        let pool = generate_pool(&catalog, true); // Music + FilmTv
        let available = filter_available(&pool, RecognitionTier::Unknown);
        // GigId(1) is Unknown tier, GigId(2) is Rising tier → only GigId(1)
        assert!(available.iter().all(|g| g.required_recognition_tier <= RecognitionTier::Unknown));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement gig_pool module**

```rust
use crate::gig::{GigCategory, GigDef};
use crate::stats::RecognitionTier;

/// Rotation A categories (odd weeks): Music, FilmTv
/// Rotation B categories (even weeks): Modeling, Variety, Endorsement, Creative
fn is_rotation_a_category(cat: GigCategory) -> bool {
    matches!(cat, GigCategory::Music | GigCategory::FilmTv)
}

/// Generate available gig pool for current rotation from the full catalog.
pub fn generate_pool(catalog: &[GigDef], rotation_a: bool) -> Vec<&GigDef> {
    catalog
        .iter()
        .filter(|g| is_rotation_a_category(g.category) == rotation_a)
        .collect()
}

/// Filter pool to only gigs the artist can access based on recognition tier.
pub fn filter_available<'a>(pool: &[&'a GigDef], artist_tier: RecognitionTier) -> Vec<&'a GigDef> {
    pool.iter()
        .filter(|g| g.is_available(artist_tier))
        .copied()
        .collect()
}
```

- [ ] **Step 4: Wire into GameState** — add `gig_catalog: Vec<GigDef>` and `available_gigs: Vec<GigDef>` fields. In `advance_week`, refresh pool every 2 weeks using `calendar.is_rotation_a()`.

- [ ] **Step 5: Run tests, commit**

```bash
git commit -m "feat(core): add bi-weekly gig pool rotation system"
```

---

## Task 2: Award System — Definitions & Judging

**Files:**
- Create: `crates/stardom-core/src/award.rs`
- Create: `data/awards/default_awards.ron`
- Modify: `crates/stardom-core/src/lib.rs`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::{BaseAttributes, ProfessionalSkills};
    use crate::persona::ImageTags;
    use crate::training::SkillTarget;

    fn model_award() -> AwardDef {
        AwardDef {
            id: AwardId(1),
            name: "Model Award".into(),
            ceremony_month: 9,
            nomination_month: 9,
            scoring_skills: vec![(SkillTarget::Poise, 1.0)],
            scoring_image_tags: vec![(ImageTag::Sexy, 0.5), (ImageTag::Cool, 0.3)],
            requires_gig_category: Some(GigCategory::Modeling),
            ai_competitor_score: 3000,
            recognition_boost: 500,
            reputation_boost: 15,
        }
    }

    #[test]
    fn award_score_calculation() {
        let award = model_award();
        let mut skills = ProfessionalSkills::default();
        skills.poise = 4000;
        let image = ImageTags { sexy: 60, cool: 40, ..Default::default() };
        // score = poise*1.0 + sexy*0.5*100 + cool*0.3*100 = 4000 + 3000 + 1200 = 8200
        let score = award.calculate_score(&skills, &image);
        assert_eq!(score, 8200);
    }

    #[test]
    fn award_nomination_requires_matching_gig() {
        let award = model_award();
        let completed_categories = vec![GigCategory::Music];
        assert!(!award.is_nominated(&completed_categories));
        let completed_categories = vec![GigCategory::Modeling];
        assert!(award.is_nominated(&completed_categories));
    }

    #[test]
    fn award_wins_when_beating_ai() {
        let award = model_award();
        assert!(award.is_winner(3500)); // > ai_competitor_score 3000
        assert!(!award.is_winner(2500)); // < 3000
    }

    #[test]
    fn award_no_category_requirement() {
        let mut award = model_award();
        award.requires_gig_category = None;
        assert!(award.is_nominated(&[])); // always nominated
    }

    #[test]
    fn serialization_roundtrip() {
        let award = model_award();
        let s = ron::to_string(&award).unwrap();
        let d: AwardDef = ron::from_str(&s).unwrap();
        assert_eq!(d.name, "Model Award");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement award module**

```rust
use serde::{Deserialize, Serialize};
use crate::attribute::ProfessionalSkills;
use crate::gig::GigCategory;
use crate::persona::{ImageTag, ImageTags};
use crate::training::SkillTarget;

// NOTE: Add AwardId to id_newtype! macro in types.rs:
// id_newtype!(ArtistId, GigId, OutfitId, CrisisId, TrainingId, JobId, AwardId);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwardDef {
    pub id: AwardId,
    pub name: String,
    pub ceremony_month: u32,     // 1-12
    pub nomination_month: u32,   // 1-12, same or earlier
    pub scoring_skills: Vec<(SkillTarget, f64)>,
    pub scoring_image_tags: Vec<(ImageTag, f64)>,
    pub requires_gig_category: Option<GigCategory>,
    pub ai_competitor_score: i32,
    pub recognition_boost: i64,
    pub reputation_boost: i32,
}

impl AwardDef {
    /// Calculate artist's award score from skills and image tags.
    /// Skills contribute directly, image tags are scaled by ×100 to match skill range.
    pub fn calculate_score(&self, skills: &ProfessionalSkills, image: &ImageTags) -> i32 {
        let skill_score: f64 = self.scoring_skills.iter()
            .map(|(target, weight)| skills.get(*target) as f64 * weight)
            .sum();
        let image_score: f64 = self.scoring_image_tags.iter()
            .map(|(tag, weight)| tag.value_from(image) as f64 * weight * 100.0)
            .sum();
        (skill_score + image_score) as i32
    }

    /// Check if artist is nominated (must have completed at least one gig in the required category).
    pub fn is_nominated(&self, completed_categories: &[GigCategory]) -> bool {
        match &self.requires_gig_category {
            None => true,
            Some(cat) => completed_categories.contains(cat),
        }
    }

    /// Check if artist's score beats the AI competitor.
    pub fn is_winner(&self, artist_score: i32) -> bool {
        artist_score > self.ai_competitor_score
    }
}
```

- [ ] **Step 4: Create `data/awards/default_awards.ron`**

```ron
[
    AwardDef(
        id: AwardId(1),
        name: "Model Award",
        ceremony_month: 9,
        nomination_month: 9,
        scoring_skills: [(Poise, 1.0)],
        scoring_image_tags: [(Sexy, 0.3), (Cool, 0.3), (Pure, 0.2)],
        requires_gig_category: Some(Modeling),
        ai_competitor_score: 3000,
        recognition_boost: 500,
        reputation_boost: 15,
    ),
    AwardDef(
        id: AwardId(2),
        name: "Music Award",
        ceremony_month: 11,
        nomination_month: 11,
        scoring_skills: [(Vocal, 0.7), (Creativity, 0.3)],
        scoring_image_tags: [],
        requires_gig_category: Some(Music),
        ai_competitor_score: 3500,
        recognition_boost: 600,
        reputation_boost: 20,
    ),
    AwardDef(
        id: AwardId(3),
        name: "Film Award",
        ceremony_month: 12,
        nomination_month: 12,
        scoring_skills: [(Acting, 0.8), (Eloquence, 0.2)],
        scoring_image_tags: [],
        requires_gig_category: Some(FilmTv),
        ai_competitor_score: 4000,
        recognition_boost: 800,
        reputation_boost: 25,
    ),
]
```

- [ ] **Step 5: Run tests, commit**

```bash
git commit -m "feat(core): add award system with scoring, nomination, and AI judging"
```

---

## Task 3: PR Crisis System — Definitions & Resolution

**Files:**
- Create: `crates/stardom-core/src/crisis.rs`
- Create: `data/crises/default_crises.ron`
- Modify: `crates/stardom-core/src/lib.rs`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn scandal_crisis() -> CrisisDef {
        CrisisDef {
            id: CrisisId(1),
            name: "Dating Scandal".into(),
            description: "Photos leaked of artist on a date.".into(),
            trigger_weight: 10,
            min_recognition_tier: RecognitionTier::Newcomer,
            choices: vec![
                CrisisChoice {
                    label: "Deny everything".into(),
                    reputation_change: -5,
                    popularity_change: 10,
                    stress_change: 15,
                    image_tag_changes: vec![],
                },
                CrisisChoice {
                    label: "Confirm and own it".into(),
                    reputation_change: 3,
                    popularity_change: 20,
                    stress_change: 5,
                    image_tag_changes: vec![(ImageTag::Sexy, 10), (ImageTag::Pure, -15)],
                },
            ],
        }
    }

    #[test]
    fn crisis_choice_count() {
        let crisis = scandal_crisis();
        assert_eq!(crisis.choices.len(), 2);
    }

    #[test]
    fn apply_crisis_choice() {
        let crisis = scandal_crisis();
        let effect = crisis.resolve(1).unwrap(); // "Confirm and own it"
        assert_eq!(effect.reputation_change, 3);
        assert_eq!(effect.popularity_change, 20);
        assert_eq!(effect.stress_change, 5);
        assert_eq!(effect.image_tag_changes.len(), 2);
    }

    #[test]
    fn crisis_resolve_out_of_bounds_defaults_to_first() {
        let crisis = scandal_crisis();
        let effect = crisis.resolve(99).unwrap();
        assert_eq!(effect.reputation_change, -5); // first choice
    }

    #[test]
    fn crisis_resolve_empty_choices_returns_none() {
        let mut crisis = scandal_crisis();
        crisis.choices.clear();
        assert!(crisis.resolve(0).is_none());
    }

    #[test]
    fn should_trigger_based_on_tier() {
        let crisis = scandal_crisis();
        assert!(!crisis.can_trigger(RecognitionTier::Unknown));
        assert!(crisis.can_trigger(RecognitionTier::Newcomer));
        assert!(crisis.can_trigger(RecognitionTier::Star));
    }

    #[test]
    fn serialization_roundtrip() {
        let crisis = scandal_crisis();
        let s = ron::to_string(&crisis).unwrap();
        let d: CrisisDef = ron::from_str(&s).unwrap();
        assert_eq!(d.name, "Dating Scandal");
        assert_eq!(d.choices.len(), 2);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement crisis module**

```rust
use serde::{Deserialize, Serialize};
use crate::persona::ImageTag;
use crate::stats::RecognitionTier;
use crate::types::CrisisId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrisisChoice {
    pub label: String,
    pub reputation_change: i32,
    pub popularity_change: i32,
    pub stress_change: i32,
    pub image_tag_changes: Vec<(ImageTag, i32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrisisDef {
    pub id: CrisisId,
    pub name: String,
    pub description: String,
    pub trigger_weight: u32,  // relative probability weight
    pub min_recognition_tier: RecognitionTier,
    pub choices: Vec<CrisisChoice>,
}

pub struct CrisisEffect {
    pub reputation_change: i32,
    pub popularity_change: i32,
    pub stress_change: i32,
    pub image_tag_changes: Vec<(ImageTag, i32)>,
}

impl CrisisDef {
    pub fn can_trigger(&self, artist_tier: RecognitionTier) -> bool {
        artist_tier >= self.min_recognition_tier
    }

    /// Resolve crisis with given choice index. Out-of-bounds defaults to first choice.
    /// Returns None if choices is empty (malformed data).
    pub fn resolve(&self, choice_index: usize) -> Option<CrisisEffect> {
        let choice = self.choices.get(choice_index).or_else(|| self.choices.first())?;
        Some(CrisisEffect {
            reputation_change: choice.reputation_change,
            popularity_change: choice.popularity_change,
            stress_change: choice.stress_change,
            image_tag_changes: choice.image_tag_changes.clone(),
        })
    }
}

/// Roll whether a crisis triggers this week.
/// base_chance: from config (e.g., 5 = 5% base per artist per week)
/// rebellion: artist's rebellion trait value
/// roll: random value in range 0..100
/// Spec A.6: Rebellion > 70 → +20% trigger rate
pub fn roll_crisis_chance(base_chance: u32, rebellion: i32, roll: u32) -> bool {
    let mut chance = base_chance;
    if rebellion > 70 {
        chance = chance * 120 / 100;
    }
    roll < chance
}
```

- [ ] **Step 4: Create `data/crises/default_crises.ron`** with 3 sample crises

- [ ] **Step 5: Run tests, commit**

```bash
git commit -m "feat(core): add PR crisis system with choices and resolution"
```

---

## Task 4: Office Upgrade System

**Files:**
- Create: `crates/stardom-core/src/office.rs`
- Create: `data/offices/default_offices.ron`
- Modify: `crates/stardom-core/src/company.rs`
- Modify: `crates/stardom-core/src/lib.rs`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::company::OfficeTier;
    use crate::types::Money;

    fn default_upgrades() -> Vec<OfficeUpgradeDef> {
        vec![
            OfficeUpgradeDef {
                tier: OfficeTier::Standard,
                cost: Money(500_000),
                max_artists_bonus: 1,
                training_cost_discount_pct: 5,
                weekly_upkeep: Money(2_000),
            },
            OfficeUpgradeDef {
                tier: OfficeTier::Premium,
                cost: Money(2_000_000),
                max_artists_bonus: 2,
                training_cost_discount_pct: 10,
                weekly_upkeep: Money(5_000),
            },
            OfficeUpgradeDef {
                tier: OfficeTier::Luxury,
                cost: Money(5_000_000),
                max_artists_bonus: 3,
                training_cost_discount_pct: 15,
                weekly_upkeep: Money(10_000),
            },
        ]
    }

    #[test]
    fn next_upgrade_from_starter() {
        let upgrades = default_upgrades();
        let next = next_upgrade(OfficeTier::Starter, &upgrades);
        assert!(next.is_some());
        assert_eq!(next.unwrap().tier, OfficeTier::Standard);
    }

    #[test]
    fn next_upgrade_from_luxury_is_none() {
        let upgrades = default_upgrades();
        let next = next_upgrade(OfficeTier::Luxury, &upgrades);
        assert!(next.is_none());
    }

    #[test]
    fn can_afford_upgrade() {
        let upgrades = default_upgrades();
        let next = next_upgrade(OfficeTier::Starter, &upgrades).unwrap();
        assert!(can_afford(Money(600_000), next));
        assert!(!can_afford(Money(400_000), next));
    }

    #[test]
    fn downgrade_returns_partial_cost() {
        let upgrades = default_upgrades();
        // Downgrading from Standard → Starter returns 40% of Standard cost
        let refund = downgrade_refund(OfficeTier::Standard, &upgrades);
        assert_eq!(refund, Money(200_000)); // 500_000 * 40%
    }

    #[test]
    fn downgrade_from_starter_returns_zero() {
        let upgrades = default_upgrades();
        let refund = downgrade_refund(OfficeTier::Starter, &upgrades);
        assert_eq!(refund, Money(0));
    }

    #[test]
    fn weekly_upkeep_for_tier() {
        let upgrades = default_upgrades();
        assert_eq!(get_weekly_upkeep(OfficeTier::Starter, &upgrades), Money(0));
        assert_eq!(get_weekly_upkeep(OfficeTier::Standard, &upgrades), Money(2_000));
    }

    #[test]
    fn serialization_roundtrip() {
        let upgrades = default_upgrades();
        let s = ron::to_string(&upgrades).unwrap();
        let d: Vec<OfficeUpgradeDef> = ron::from_str(&s).unwrap();
        assert_eq!(d.len(), 3);
        assert_eq!(d[0].tier, OfficeTier::Standard);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement office module**

```rust
use serde::{Deserialize, Serialize};
use crate::company::OfficeTier;
use crate::types::Money;

const DOWNGRADE_REFUND_PCT: i64 = 40;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficeUpgradeDef {
    pub tier: OfficeTier,
    pub cost: Money,
    pub max_artists_bonus: u32,
    pub training_cost_discount_pct: u32,
    pub weekly_upkeep: Money,
}

/// Returns the next upgrade tier above current. Assumes upgrades slice is sorted by tier (ascending).
pub fn next_upgrade<'a>(current: OfficeTier, upgrades: &'a [OfficeUpgradeDef]) -> Option<&'a OfficeUpgradeDef> {
    upgrades.iter().find(|u| u.tier > current)
}

pub fn can_afford(balance: Money, upgrade: &OfficeUpgradeDef) -> bool {
    balance.0 >= upgrade.cost.0
}

/// Returns 40% of current tier's cost as refund, or 0 if at Starter.
pub fn downgrade_refund(current: OfficeTier, upgrades: &[OfficeUpgradeDef]) -> Money {
    upgrades
        .iter()
        .find(|u| u.tier == current)
        .map(|u| Money(u.cost.0 * DOWNGRADE_REFUND_PCT / 100))
        .unwrap_or(Money(0))
}

pub fn get_weekly_upkeep(current: OfficeTier, upgrades: &[OfficeUpgradeDef]) -> Money {
    upgrades
        .iter()
        .find(|u| u.tier == current)
        .map(|u| u.weekly_upkeep)
        .unwrap_or(Money(0))
}
```

- [ ] **Step 4: Create `data/offices/default_offices.ron`**

```ron
[
    OfficeUpgradeDef(
        tier: Standard,
        cost: Money(500000),
        max_artists_bonus: 1,
        training_cost_discount_pct: 5,
        weekly_upkeep: Money(2000),
    ),
    OfficeUpgradeDef(
        tier: Premium,
        cost: Money(2000000),
        max_artists_bonus: 2,
        training_cost_discount_pct: 10,
        weekly_upkeep: Money(5000),
    ),
    OfficeUpgradeDef(
        tier: Luxury,
        cost: Money(5000000),
        max_artists_bonus: 3,
        training_cost_discount_pct: 15,
        weekly_upkeep: Money(10000),
    ),
]
```

- [ ] **Step 5: Add `UpgradeOffice` and `DowngradeOffice` commands to game.rs** (implementation in Task 5)

- [ ] **Step 6: Run tests, commit**

```bash
git commit -m "feat(core): add office upgrade system with costs, bonuses, and downgrade refund"
```

---

## Task 5: Wire Everything Into the Game Loop

**Files:**
- Modify: `crates/stardom-core/src/game.rs`
- Modify: `crates/stardom-core/src/config.rs`

This is the integration task. Extend `GameState`, `GameCommand`, and `advance_week` to:
1. Track `gig_catalog` and refresh `available_gigs` bi-weekly
2. Track `awards_won` per artist, check award ceremonies by month
3. Roll for crisis events each week, store `active_crisis` for player to respond
4. Deduct weekly office upkeep
5. Add `UpgradeOffice` / `DowngradeOffice` / `RespondToCrisis` commands

- [ ] **Step 1: Write failing tests**

```rust
// Add to game.rs tests:

fn game_with_office_data() -> GameState {
    let mut game = default_game();
    game.office_upgrades = vec![
        OfficeUpgradeDef { tier: OfficeTier::Standard, cost: Money(500_000), max_artists_bonus: 1, training_cost_discount_pct: 5, weekly_upkeep: Money(2_000) },
        OfficeUpgradeDef { tier: OfficeTier::Premium, cost: Money(2_000_000), max_artists_bonus: 2, training_cost_discount_pct: 10, weekly_upkeep: Money(5_000) },
        OfficeUpgradeDef { tier: OfficeTier::Luxury, cost: Money(5_000_000), max_artists_bonus: 3, training_cost_discount_pct: 15, weekly_upkeep: Money(10_000) },
    ];
    game
}

#[test]
fn office_upgrade_deducts_cost_and_increases_tier() {
    let mut game = game_with_office_data();
    game.process_command(GameCommand::UpgradeOffice);
    // Should fail — not enough money? Actually 1M >= 500K, so it succeeds
    assert_eq!(game.company.office_tier, OfficeTier::Standard);
    assert_eq!(game.company.balance, Money(1_000_000 - 500_000));
}

#[test]
fn office_downgrade_refunds_40_pct() {
    let mut game = game_with_office_data();
    game.process_command(GameCommand::UpgradeOffice); // Starter → Standard, -500K
    game.process_command(GameCommand::DowngradeOffice); // Standard → Starter, +200K
    assert_eq!(game.company.office_tier, OfficeTier::Starter);
    assert_eq!(game.company.balance, Money(1_000_000 - 500_000 + 200_000));
}

#[test]
fn weekly_upkeep_deducted() {
    let mut game = game_with_office_data();
    game.process_command(GameCommand::UpgradeOffice); // Standard tier, upkeep 2000/week
    let balance_after_upgrade = game.company.balance;
    game.process_command(GameCommand::AdvanceWeek);
    assert_eq!(game.company.balance, balance_after_upgrade - Money(2_000));
}

#[test]
fn crisis_respond_applies_effects() {
    let mut game = default_game();
    let mut artist = make_artist_with_popularity(50);
    artist.stats.recognition = 500; // Newcomer tier
    game.artists.push(artist);
    // Manually inject a crisis
    let crisis = CrisisDef {
        id: CrisisId(1), name: "Scandal".into(), description: "Test".into(),
        trigger_weight: 100, min_recognition_tier: RecognitionTier::Unknown,
        choices: vec![
            CrisisChoice {
                label: "Deny".into(), reputation_change: -5,
                popularity_change: 10, stress_change: 15,
                image_tag_changes: vec![],
            },
        ],
    };
    game.active_crises.push((0, crisis));
    game.process_command(GameCommand::RespondToCrisis { crisis_index: 0, choice: 0 });
    assert!(game.active_crises.is_empty());
    assert_eq!(game.artists[0].stats.reputation, -5);
    assert_eq!(game.artists[0].stats.popularity, 60); // 50 + 10
    assert_eq!(game.artists[0].stats.stress, 15);
}
```

- [ ] **Step 2: Run tests to verify they fail**

- [ ] **Step 3: Implement game loop integration**

Add to `GameState`:
```rust
pub gig_catalog: Vec<GigDef>,
pub available_gigs: Vec<GigDef>,
pub award_defs: Vec<AwardDef>,
pub awards_won: Vec<(ArtistId, AwardId)>,
pub completed_gig_categories: Vec<(ArtistId, GigCategory)>, // track for award nomination
pub crisis_catalog: Vec<CrisisDef>,
pub active_crises: Vec<(usize, CrisisDef)>, // (artist_index, crisis)
pub office_upgrades: Vec<OfficeUpgradeDef>,
```

All with `#[serde(default)]`.

Add to `GameCommand`:
```rust
UpgradeOffice,
DowngradeOffice,
RespondToCrisis { crisis_index: usize, choice: usize },
```

Add to `config.rs`:
```rust
pub crisis_base_chance: u32,  // default: 5 (5% per artist per week)
```

Update `advance_week`:
```rust
// After existing logic, add:

// 1. Deduct weekly office upkeep
let upkeep = office::get_weekly_upkeep(self.company.office_tier, &self.office_upgrades);
if upkeep.0 > 0 { self.company.spend(upkeep); }

// 2. Refresh gig pool on bi-weekly rotation
if self.calendar.week % 2 == 1 {
    self.available_gigs = gig_pool::generate_pool(&self.gig_catalog, self.calendar.is_rotation_a())
        .into_iter().cloned().collect();
}

// 3. Check award ceremonies by month
let month = self.calendar.approximate_month();
// (award ceremony check — only fire once per award per year)

// 4. Roll for crisis events (per artist)
// Use rand or accept a seed for testability
```

- [ ] **Step 4: Run tests**

- [ ] **Step 5: Run full test suite**

Run: `cargo test -p stardom-core`
Expected: all tests PASS

- [ ] **Step 6: Commit**

```bash
git commit -m "feat(core): integrate gig pool, awards, crisis, and office upgrades into game loop"
```

---

## Task 6: Integration Test & Cleanup

**Files:**
- Modify: `crates/stardom-core/src/lib.rs`

- [ ] **Step 1: Update integration test**

Add a test that:
- Creates a game with catalog data (gigs, awards, crises, offices)
- Plays through multiple weeks with training and gig assignments
- Verifies gig pool refreshes
- Upgrades office
- Verifies weekly upkeep deduction
- Responds to an injected crisis

- [ ] **Step 2: Run clippy and fmt**

Run: `cargo fmt --all && cargo clippy --workspace -- -D warnings`

- [ ] **Step 3: Run full test suite**

Run: `cargo test --workspace`

- [ ] **Step 4: Commit**

```bash
git commit -m "test(core): add Phase 3 integration test covering gig pool, awards, crisis, office upgrades"
```

---

## Phase 3 Completion Checklist

- [ ] `cargo test --workspace` — all tests pass
- [ ] `cargo clippy --workspace -- -D warnings` — clean
- [ ] `cargo fmt --check --all` — clean
- [ ] Gig pool rotates bi-weekly (rotation A: Music/FilmTv, rotation B: other categories)
- [ ] Awards: scoring formula, nomination check, AI judging, 3 default awards
- [ ] PR crisis: data-driven definitions, multi-choice resolution, rebellion-boosted trigger rate
- [ ] Office upgrades: 4 tiers, costs, bonuses, downgrade refund at 40%, weekly upkeep
- [ ] Game loop processes all new systems each week/month

---

## What Phase 4 Will Cover

> Not part of this plan. Listed for context only.

- Costume/outfit system (Image Tag modifiers)
- Mini-game framework
- Artist recruitment system (scouting, dialogue, signing)
- Narrative/scripting engine framework
- Save/load system
