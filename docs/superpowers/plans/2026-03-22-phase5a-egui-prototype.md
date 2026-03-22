# Phase 5A: Playable Prototype — egui UI Integration

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire stardom-core's GameState into the Bevy app with a functional egui-based prototype UI, making all game systems playable — manage artists, assign activities, advance weeks, purchase outfits, respond to crises, upgrade office, and progress toward awards.

**Architecture:** The Bevy app uses `bevy_egui` for all UI. GameState is stored as a Bevy `Resource`. UI is organized into screens via Bevy `States` (MainMenu → InGame). The InGame screen has tabbed panels for different management views. All interaction goes through `GameCommand` processing. This is a developer/prototype UI that will be replaced by pixel art UI in Phase 5B.

**Tech Stack:** Rust 1.94, Bevy 0.18, bevy_egui 0.39, stardom-core

**Spec reference:** `docs/superpowers/specs/2026-03-22-redo-stardom-design.md` (Section 2.2 — Presentation Layer)

---

## File Structure

```
crates/redo-stardom/
├── Cargo.toml              # (modify) add bevy_egui dependency
└── src/
    ├── main.rs             # (rewrite) app setup with states, plugins, GameState resource
    ├── states.rs           # AppState enum, state transitions
    ├── game_bridge.rs      # GameState as Bevy Resource, command processing
    ├── ui/
    │   ├── mod.rs          # UI plugin registration
    │   ├── main_menu.rs    # Main menu screen (New Game, Load, Quit)
    │   ├── hud.rs          # Top bar: calendar, money, phase indicator
    │   ├── dashboard.rs    # Company overview: artist roster, office status
    │   ├── artist_panel.rs # Artist detail: stats, skills radar, outfit, activity
    │   ├── schedule.rs     # Weekly activity assignment (training/job/gig/rest)
    │   ├── gig_market.rs   # Available gigs panel (current rotation)
    │   ├── shop.rs         # Outfit shop, office upgrade
    │   └── events.rs       # Crisis response, award ceremony, narrative dialogue
    └── data_loading.rs     # Load RON catalogs at startup
```

---

## Task 1: Add bevy_egui & App Structure

**Files:**
- Modify: `crates/redo-stardom/Cargo.toml`
- Rewrite: `crates/redo-stardom/src/main.rs`
- Create: `crates/redo-stardom/src/states.rs`
- Create: `crates/redo-stardom/src/game_bridge.rs`

- [ ] **Step 1: Add bevy_egui dependency**

```toml
[dependencies]
stardom-core = { path = "../stardom-core" }
bevy = "0.18"
bevy_egui = "0.39"
```

- [ ] **Step 2: Create states.rs**

```rust
use bevy::prelude::*;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
}
```

- [ ] **Step 3: Create game_bridge.rs**

```rust
use bevy::prelude::*;
use stardom_core::config::Settings;
use stardom_core::game::{GameCommand, GameState};

#[derive(Resource)]
pub struct GameWorld(pub GameState);

impl GameWorld {
    pub fn new_game() -> Self {
        Self(GameState::new(Settings::default()))
    }

    pub fn command(&mut self, cmd: GameCommand) {
        self.0.process_command(cmd);
    }
}
```

- [ ] **Step 4: Rewrite main.rs**

```rust
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::EguiPlugin;

mod states;
mod game_bridge;
mod ui;
mod data_loading;

use states::AppState;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "REDÓ Stardom".to_string(),
                resolution: WindowResolution::new(800, 600),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .init_state::<AppState>()
        .add_plugins(ui::UiPlugin)
        .run();
}
```

- [ ] **Step 5: Create ui/mod.rs stub**

```rust
use bevy::prelude::*;

pub mod main_menu;
pub mod hud;
pub mod dashboard;
pub mod artist_panel;
pub mod schedule;
pub mod gig_market;
pub mod shop;
pub mod events;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            main_menu::MainMenuPlugin,
        ));
    }
}
```

Create empty stub files for each ui submodule.

- [ ] **Step 6: Verify it compiles and runs**

Run: `cargo run -p redo-stardom`
Expected: window opens with egui available (no visible UI yet, just the empty window)

- [ ] **Step 7: Commit**

```bash
git commit -m "feat(app): add bevy_egui, app states, game bridge resource"
```

---

## Task 2: Main Menu Screen

**Files:**
- Implement: `crates/redo-stardom/src/ui/main_menu.rs`
- Modify: `crates/redo-stardom/src/ui/mod.rs`

- [ ] **Step 1: Implement main menu with egui**

```rust
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::game_bridge::GameWorld;
use crate::states::AppState;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, main_menu_ui.run_if(in_state(AppState::MainMenu)));
    }
}

fn main_menu_ui(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(150.0);
            ui.heading(egui::RichText::new("REDÓ Stardom").size(48.0));
            ui.add_space(40.0);

            if ui.button(egui::RichText::new("New Game").size(24.0)).clicked() {
                commands.insert_resource(GameWorld::new_game());
                next_state.set(AppState::InGame);
            }
            ui.add_space(10.0);
            if ui.button(egui::RichText::new("Quit").size(24.0)).clicked() {
                std::process::exit(0);
            }
        });
    });
}
```

- [ ] **Step 2: Register in UiPlugin**

- [ ] **Step 3: Verify — run app, see main menu, click New Game**

- [ ] **Step 4: Commit**

```bash
git commit -m "feat(app): add main menu screen with New Game button"
```

---

## Task 3: HUD — Top Bar Info Display

**Files:**
- Implement: `crates/redo-stardom/src/ui/hud.rs`
- Modify: `crates/redo-stardom/src/ui/mod.rs`

- [ ] **Step 1: Implement HUD**

Shows: Year/Week, Company Balance, Office Tier, Game Phase, Artist Count. Also has "Advance Week" button.

```rust
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use stardom_core::game::GameCommand;
use crate::game_bridge::GameWorld;
use crate::states::AppState;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, hud_ui.run_if(in_state(AppState::InGame)));
    }
}

fn hud_ui(mut contexts: EguiContexts, mut game: ResMut<GameWorld>) {
    egui::TopBottomPanel::top("hud").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            let g = &game.0;
            ui.label(format!("Year {} / Week {}", g.calendar.year, g.calendar.week));
            ui.separator();
            ui.label(format!("${}", g.company.balance.0));
            ui.separator();
            ui.label(format!("Office: {:?}", g.company.office_tier));
            ui.separator();
            ui.label(format!("Artists: {}", g.artists.len()));
            ui.separator();
            ui.label(format!("Phase: {:?}", g.phase));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⏩ Advance Week").clicked() {
                    game.command(GameCommand::AdvanceWeek);
                }
            });
        });
    });
}
```

- [ ] **Step 2: Register HudPlugin, verify**

- [ ] **Step 3: Commit**

```bash
git commit -m "feat(app): add HUD with calendar, balance, and advance week button"
```

---

## Task 4: Dashboard — Artist Roster & Company Overview

**Files:**
- Implement: `crates/redo-stardom/src/ui/dashboard.rs`
- Modify: `crates/redo-stardom/src/ui/mod.rs`

- [ ] **Step 1: Implement dashboard**

Left panel: list of signed artists with key stats. Right panel: company info.

```rust
fn dashboard_ui(mut contexts: EguiContexts, game: Res<GameWorld>, mut selected: Local<Option<usize>>) {
    egui::SidePanel::left("roster").min_width(250.0).show(contexts.ctx_mut(), |ui| {
        ui.heading("Artists");
        if game.0.artists.is_empty() {
            ui.label("No artists signed. Visit the Recruitment tab.");
        }
        for (i, artist) in game.0.artists.iter().enumerate() {
            let label = format!("{} (Age {})", artist.name, artist.age);
            if ui.selectable_label(*selected == Some(i), label).clicked() {
                *selected = Some(i);
            }
        }
    });
}
```

- [ ] **Step 2: Register, verify**

- [ ] **Step 3: Commit**

```bash
git commit -m "feat(app): add dashboard with artist roster panel"
```

---

## Task 5: Artist Detail Panel — Stats, Skills, Activity Assignment

**Files:**
- Implement: `crates/redo-stardom/src/ui/artist_panel.rs`
- Implement: `crates/redo-stardom/src/ui/schedule.rs`

This is the most important gameplay panel. Shows selected artist's full stats and lets player assign weekly activities.

- [ ] **Step 1: Implement artist detail view**

Display: base attributes (4), professional skills (6) as progress bars, inner traits, personality spectrums, image tags, aux stats (recognition tier, reputation, popularity, stress), current activity, equipped outfit.

- [ ] **Step 2: Implement activity assignment**

Training dropdown (from available training defs), Job dropdown, Gig dropdown (from current gig pool), Rest button. Each sends the corresponding GameCommand.

- [ ] **Step 3: Verify — select artist, see stats, assign training**

- [ ] **Step 4: Commit**

```bash
git commit -m "feat(app): add artist detail panel with stats display and activity assignment"
```

---

## Task 6: Gig Market & Shop Panels

**Files:**
- Implement: `crates/redo-stardom/src/ui/gig_market.rs`
- Implement: `crates/redo-stardom/src/ui/shop.rs`

- [ ] **Step 1: Implement gig market**

Shows available gigs from current rotation pool. Filter by artist's recognition tier. Click to assign.

- [ ] **Step 2: Implement shop**

Outfit purchase list. Office upgrade button. Shows costs and current state.

- [ ] **Step 3: Verify**

- [ ] **Step 4: Commit**

```bash
git commit -m "feat(app): add gig market and shop panels"
```

---

## Task 7: Events Panel — Crisis Response & Narrative Dialogue

**Files:**
- Implement: `crates/redo-stardom/src/ui/events.rs`

- [ ] **Step 1: Implement crisis response UI**

When `game.active_crises` is not empty, show a modal-like panel with crisis description and choice buttons. Sends `GameCommand::RespondToCrisis`.

- [ ] **Step 2: Implement narrative dialogue UI**

Simple dialogue box showing speaker, text, and choice buttons. Drives `ScriptRunner`.

- [ ] **Step 3: Verify**

- [ ] **Step 4: Commit**

```bash
git commit -m "feat(app): add crisis response and narrative dialogue UI"
```

---

## Task 8: Data Loading & Sample Content

**Files:**
- Implement: `crates/redo-stardom/src/data_loading.rs`
- Modify: `crates/redo-stardom/src/game_bridge.rs`

Load all RON data files at game start and populate GameState catalogs.

- [ ] **Step 1: Implement data loader**

```rust
use std::fs;
use stardom_core::gig::GigDef;
use stardom_core::training::TrainingDef;
use stardom_core::job::JobDef;
use stardom_core::award::AwardDef;
use stardom_core::crisis::CrisisDef;
use stardom_core::office::OfficeUpgradeDef;
use stardom_core::outfit::OutfitDef;
use stardom_core::recruitment::ArtistProspect;

pub fn load_all_data(game: &mut GameState) {
    // Load each catalog from data/ directory
    if let Ok(data) = fs::read_to_string("data/training/default_training.ron") {
        if let Ok(defs) = ron::from_str::<Vec<TrainingDef>>(&data) {
            // store in game or a separate resource
        }
    }
    // ... repeat for gigs, jobs, awards, crises, offices, outfits, prospects
}
```

- [ ] **Step 2: Create sample prospect data** `data/artists/default_prospects.ron`

3-5 recruitable artists with location/day/commission data.

- [ ] **Step 3: Wire into GameWorld::new_game()**

- [ ] **Step 4: Verify — start new game, see gigs in market, prospects available**

- [ ] **Step 5: Commit**

```bash
git commit -m "feat(app): add data loading from RON files and sample prospect data"
```

---

## Task 9: Recruitment UI & Full Gameplay Loop Test

**Files:**
- Add recruitment tab to dashboard or as separate panel
- Manual gameplay verification

- [ ] **Step 1: Add recruitment panel**

Shows available prospects (based on current week/day). Click to negotiate and sign.

- [ ] **Step 2: Full gameplay loop manual test**

Verify the complete loop: Start game → Sign artist from prospects → Assign training → Advance week → Assign gig → Advance weeks → Gig completes → Buy outfit → Equip → Respond to crisis (if triggered) → Check award at ceremony month → Continue or save.

- [ ] **Step 3: Commit**

```bash
git commit -m "feat(app): add recruitment UI, complete gameplay loop"
```

---

## Task 10: Final Cleanup & Push

- [ ] **Step 1: Run clippy and fmt**

```bash
cargo fmt --all && cargo clippy --workspace -- -D warnings
```

- [ ] **Step 2: Run all tests**

```bash
cargo test --workspace
```

- [ ] **Step 3: Verify app runs end-to-end**

```bash
cargo run -p redo-stardom
```

- [ ] **Step 4: Commit and push**

```bash
git commit -m "chore: Phase 5A cleanup"
git push
```

---

## Phase 5A Completion Checklist

- [ ] Window opens with main menu (New Game / Quit)
- [ ] New Game creates a GameState and transitions to InGame
- [ ] HUD shows calendar, balance, office tier, phase
- [ ] Advance Week button progresses time
- [ ] Artist roster displays signed artists
- [ ] Artist detail shows all 5 attribute layers + aux stats
- [ ] Can assign Training / Job / Gig / Rest to artists
- [ ] Gig market shows bi-weekly rotation pool
- [ ] Shop allows outfit purchase and office upgrade
- [ ] Crisis response modal works
- [ ] Recruitment panel lets player sign new artists
- [ ] Data loads from RON files
- [ ] Save/Load works (at minimum save-to-string for debugging)
- [ ] All stardom-core tests still pass

---

## What Phase 5B Will Cover

> Not part of this plan. Listed for context only.

- Replace egui with PC-98 pixel art Bevy native UI
- Custom font rendering for CJK characters
- Sprite-based artist portraits and backgrounds
- Animation system for screen transitions
- Audio integration (BGM, SFX)
