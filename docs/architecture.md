# REDÓ Stardom — Architecture Overview

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Presentation Layer (crates/redo-stardom)                │
│                                                         │
│  Bevy 0.18 + bevy_egui 0.39                            │
│                                                         │
│  ┌──────────┐ ┌──────────┐ ┌────────────┐              │
│  │ main.rs  │ │ states.rs│ │game_bridge │              │
│  │ App setup│ │ AppState │ │ GameWorld  │              │
│  └──────────┘ └──────────┘ └─────┬──────┘              │
│                                   │                     │
│  ┌── ui/ ─────────────────────────┼───────────────┐    │
│  │ hud        → TopPanel          │               │    │
│  │ game_log   → BottomPanel       │               │    │
│  │ dashboard  → SidePanel(left)   │               │    │
│  │ central_tabs → CentralPanel    │               │    │
│  │   tab 0: Artist Detail         │               │    │
│  │   tab 1: Gig Market            │               │    │
│  │   tab 2: Shop                  │               │    │
│  │   tab 3: Recruitment           │               │    │
│  │ events     → Crisis Modal      │               │    │
│  │ week_report → Report Modal     │               │    │
│  │ week_plan  → WeekPlan Resource │               │    │
│  │ display    → Chinese text helpers              │    │
│  └────────────────────────────────────────────────┘    │
│                         │                               │
│                    GameCommand                          │
│                         ↓                               │
├─────────────────────────────────────────────────────────┤
│  Game Core (crates/stardom-core)                        │
│                                                         │
│  Pure Rust library — zero Bevy dependency               │
│                                                         │
│  ┌─ Data Models ──────────────────────────────────┐    │
│  │ types.rs      ID newtypes, Money, Activity     │    │
│  │ attribute.rs  BaseAttributes (4)               │    │
│  │               ProfessionalSkills (6)           │    │
│  │               InnerTraits (2)                  │    │
│  │ persona.rs    PersonalitySpectrums (4 axes)    │    │
│  │               ImageTags (6), ImageTag enum     │    │
│  │ stats.rs      AuxiliaryStats, RecognitionTier  │    │
│  │ artist.rs     Artist (combines all above)      │    │
│  │ company.rs    CompanyState, OfficeTier          │    │
│  │ calendar.rs   Calendar, week/year tracking     │    │
│  │ config.rs     Settings (from TOML)             │    │
│  └────────────────────────────────────────────────┘    │
│                                                         │
│  ┌─ Activity Systems ────────────────────────────┐     │
│  │ training.rs   TrainingDef, efficiency formula  │     │
│  │ job.rs        JobDef, tier-gated availability  │     │
│  │ gig.rs        GigDef, success score, pay calc  │     │
│  │ gig_pool.rs   Bi-weekly rotation pool          │     │
│  │ scheduling.rs Apply effects to artists         │     │
│  └────────────────────────────────────────────────┘    │
│                                                         │
│  ┌─ Event Systems ───────────────────────────────┐     │
│  │ award.rs      AwardDef, scoring, judging       │     │
│  │ crisis.rs     CrisisDef, choices, resolution   │     │
│  │ office.rs     OfficeUpgradeDef, costs, bonuses │     │
│  │ outfit.rs     OutfitDef, image tag modifiers   │     │
│  │ recruitment.rs ArtistProspect, commission      │     │
│  │ narrative.rs  ScriptDef, DialogueNode, Runner  │     │
│  └────────────────────────────────────────────────┘    │
│                                                         │
│  ┌─ Core ────────────────────────────────────────┐     │
│  │ game.rs       GameState, GameCommand,          │     │
│  │               advance_week(), process_command()│     │
│  │ data_loader.rs ArtistDefinition, RON loading   │     │
│  │ save.rs       Save/Load (RON serialization)    │     │
│  └────────────────────────────────────────────────┘    │
│                                                         │
└─────────────────────────────────────────────────────────┘

┌─ Data Files (data/) ────────────────────────────────────┐
│ artists/   training/   jobs/   gigs/   awards/          │
│ crises/    offices/    outfits/   scripts/              │
│ All RON format, loaded at game start                    │
└─────────────────────────────────────────────────────────┘
```

## Attribute System (5 Layers)

```
┌─ Base Attributes (4) ─── Innate talent, diamond radar
│   Stamina / Intellect / Empathy / Charm
│   Range: 1-100, slow growth, affects training efficiency
│
├─ Professional Skills (6) ─── Growth focus, hexagonal radar
│   Vocal / Acting / Dance / Poise / Eloquence / Creativity
│   Range: 0-10,000, grows through training/jobs/gigs
│
├─ Inner Traits (2) ─── Double-edged swords
│   Confidence / Rebellion
│   Range: 0-100, both extremes have pros and cons
│
├─ Personality Spectrums (4) ─── Bipolar axes, event-driven drift
│   Introvert↔Extrovert / Intuitive↔Logical
│   Cautious↔Adventurous / Easygoing↔Competitive
│   Range: -100 to +100, affects activity fit
│
├─ Image Tags (6) ─── Independent, can coexist
│   Pure / Sexy / Cool / Intellectual / Funny / Mysterious
│   Range: 0-100, affected by outfits/gigs/events/age
│
└─ Auxiliary Stats (4) ─── Market status dashboard
    Recognition: 0→∞ (only increases, tiered: Unknown→Superstar)
    Reputation: -100↔+100 (both extremes viable)
    Popularity: 0-100 (decays weekly, needs maintenance)
    Stress: 0-100 (too high = negative events)
```

## Game Loop (Per Week)

```
Player assigns activities (WeekPlan)
         │
         ▼
    ┌─────────────┐
    │ All assigned?│──no──→ (wait)
    └──────┬──────┘
           │ yes
           ▼
    execute_week()
    ├── Snapshot artist stats
    ├── Send GameCommands (training/job/gig/rest)
    ├── GameCommand::AdvanceWeek
    │   ├── Decrement gig lock timers
    │   ├── Complete finished gigs → rewards
    │   ├── Age artists (on year rollover)
    │   ├── Age-based image decay (Pure, age > 25)
    │   ├── Popularity decay (base + inactivity)
    │   ├── Reset non-locked activities to Idle
    │   ├── Deduct office upkeep
    │   ├── Gig pool rotation (odd weeks)
    │   ├── Crisis random trigger (per artist)
    │   ├── Award ceremony (by month)
    │   ├── Bankruptcy check
    │   └── Phase transition check
    ├── Diff before/after → WeekReport
    └── Show report modal → player clicks "確定"
```

## Data Flow

```
RON files (data/)
    │
    ▼ (loaded at game start)
GameState fields:
    gig_catalog, office_upgrades, outfit_catalog,
    crisis_catalog, award_defs, prospects
    │
    ▼ (also loaded separately)
GameCatalogs (Bevy Resource):
    training: Vec<TrainingDef>
    jobs: Vec<JobDef>
    │
    ▼ (player interaction)
WeekPlan (Bevy Resource):
    assignments: HashMap<artist_index, PlannedActivity>
    │
    ▼ (on advance week)
GameCommand → GameState mutation
    │
    ▼ (after advance)
WeekReport (Bevy Resource, temporary):
    entries, total_income, total_expenses
    │
    ▼ (player dismisses)
WeekReport removed, next week begins
```

## Module Dependency Graph (stardom-core)

```
types ←── attribute ←── persona
  ↑          ↑             ↑
  │          │             │
  ├── stats ─┤             │
  │          │             │
  ├── artist ┘─────────────┘
  │     ↑
  │     ├── training
  │     ├── job (uses training::SkillTarget)
  │     ├── gig (uses training::SkillTarget, persona::ImageTag)
  │     ├── scheduling (uses training, job, gig)
  │     ├── gig_pool (uses gig)
  │     ├── award (uses training::SkillTarget, persona::ImageTag)
  │     ├── crisis (uses persona::ImageTag)
  │     ├── office (uses company::OfficeTier)
  │     ├── outfit (uses persona::ImageTag)
  │     ├── recruitment (uses data_loader)
  │     ├── narrative
  │     └── minigame (uses training::SkillTarget)
  │
  └── game (depends on ALL above)
       ↑
       └── save (generic, works with any Serialize type)
```
