# Redo Stardom — Game Design Specification

> A modern reimagining of the classic 明星志願 (Stardom) series.
> All code, assets, and content are original to avoid copyright issues.

---

## 1. Game Overview

### 1.1 Concept

A star-management simulation game inspired by 明星志願 1 & 2, set in a fictional world that freely blends elements from various eras of the entertainment industry — from traditional TV/film/music to modern phenomena like idol groups, variety shows, and personal content creation.

### 1.2 Core Experience

- **Primary mode (v1):** Manager mode — the player runs a talent agency, scouting and developing artists toward stardom
- **Future mode:** Artist mode — the player IS the artist, managing their own career
- Both modes share the same simulation engine; the difference is perspective and UI

### 1.3 Win Conditions & Time Structure

| Phase | Description |
|-------|-------------|
| **Main game** | 3-year goal period. Default objective: win three major awards with a **single artist** (the "ace" of your agency). Objectives are data-driven and extensible (e.g., save company from financial crisis). In Artist mode, the player character must win all three personally. |
| **Post-ending** | Player may continue playing indefinitely after reaching an ending (similar to Civilization's "One More Turn") |
| **Retirement** | Artist age limit triggers mandatory retirement from active performing (default: age 40, data-configurable). In Manager mode, the player (manager) does not retire, but artists do — losing all artists ends the run. Starting age for artists is defined per-artist in data files (typical range: 16–25). |

### 1.4 Artist Roster

- Default cap: **3 artists** simultaneously under contract
- Architecture supports expanding beyond 3
- Cap can increase as the company grows (unlockable via milestones)

---

## 2. Technical Architecture

### 2.1 Tech Stack

- **Language:** Rust
- **Game framework:** Bevy (rendering, UI, audio, input)
- **Target platform:** Desktop (native)

### 2.2 Two-Layer Architecture (Approach B)

The game is split into two distinct layers:

```
┌─────────────────────────────────────┐
│  Presentation Layer (Bevy)          │
│  - PC-98 style fine pixel rendering │
│  - UI, animations, audio            │
│  - Input handling → Commands        │
│  - Reads from GameState to display  │
├─────────────────────────────────────┤
│  Game Core (Pure Rust library)      │
│  - Simulation engine (time, events) │
│  - Data models (artists, gigs, etc) │
│  - Scripting engine (events/story)  │
│  - Data loader (RON/TOML)           │
└─────────────────────────────────────┘
```

**Rationale:**
- Game logic is independently testable via `cargo test` without running Bevy
- Data-driven design naturally fits this separation
- Mod system is easier to implement on a clean data layer
- Manager mode and Artist mode are just different presentation views over the same simulation
- UI framework can be swapped or augmented (e.g., egui) without touching game logic

### 2.3 Data-Driven Design & Mod Support

- All game content (artists, gigs, events, scripts, awards, items) defined in external data files (RON/TOML)
- Game core loads and validates data at startup
- Future: player-facing mod editor UI for creating custom content (artists, events, endings, gigs)
- Story/narrative content is also a data module — the scripting engine is built into v1, content is incrementally added

### 2.4 Visual Style

**PC-98 fine pixel art** — the refined, high-color-count pixel style of Japanese PC-9801 games, NOT the chunky low-res pixel art common in modern indie games. This matches the tonal elegance of the original Stardom series.

---

## 3. Attribute System

The attribute system is organized into 5 layers, each serving a distinct gameplay role.

### 3.1 Layer Overview

```
┌─ Base Attributes (4) ─── Innate talent, diamond radar
│   Stamina / Intellect / Empathy / Charm
│
├─ Professional Skills (6) ─── Growth focus, hexagonal radar
│   Vocal / Acting / Dance / Poise / Eloquence / Creativity
│
├─ Inner Traits (2) ─── Double-edged, strategic trade-offs
│   Confidence / Rebellion
│
├─ Personality Spectrums (4) ─── Bipolar axes, shift via events
│   Introvert↔Extrovert / Intuitive↔Logical / Cautious↔Adventurous / Easygoing↔Competitive
│
├─ Image Tags (6) ─── Independent values, coexist freely, define public persona
│   Pure / Sexy / Cool / Intellectual / Funny / Mysterious
│
└─ Auxiliary Stats (4) ─── Market status dashboard
    Recognition / Reputation / Popularity / Stress
```

### 3.2 Base Attributes (4) — Innate Talent

These represent an artist's natural foundation. They affect training efficiency for professional skills and are relatively slow to change. Visualized as a **diamond radar chart**.

| Attribute | Key | Description | Affects Training Efficiency Of |
|-----------|-----|-------------|-------------------------------|
| Stamina | STA | Physical fitness, endurance, coordination | Dance, Acting (action scenes) |
| Intellect | INT | Learning capacity, comprehension, analysis | Eloquence, Creativity |
| Empathy | EMP | Emotional sensitivity, resonance, depth | Acting, Vocal (emotional delivery) |
| Charm | CHA | Natural attractiveness, presence, aura | Poise, Popularity growth baseline |

**Mapping to professional skills:**

```
Stamina  ──→ Dance (primary), Acting (secondary)
Intellect ──→ Eloquence (primary), Creativity (primary)
Empathy  ──→ Acting (primary), Vocal (primary)
Charm    ──→ Poise (primary), Eloquence (secondary)
```

**Numerical model:**
- Range: 1–100
- Set at artist creation (influenced by background/origin data)
- Grows very slowly through specific life events, not through regular training
- Primarily a multiplier on skill training speed, not a direct performance stat

### 3.3 Professional Skills (6) — Career Growth

These are the core skills that players actively develop through training, part-time jobs, and gigs. Visualized as a **hexagonal radar chart** — the primary visual indicator of artist growth.

| Skill | Key | Description | Career Paths |
|-------|-----|-------------|-------------|
| Vocal | VOC | Singing technique, pitch, range, emotional delivery | Singer, music-related gigs |
| Acting | ACT | Dramatic performance, character portrayal | Actor, film/TV/short drama |
| Dance | DAN | Dance technique, body expression, rhythm | Dancer, idol group, MV performance |
| Poise | POI | Posture, stage presence, camera appeal | Model, endorsements, fashion |
| Eloquence | ELO | Speaking skill, humor, quick wit, improvisation | Host, variety shows, interviews, live content |
| Creativity | CRE | Songwriting, scriptwriting, content planning | Self-produced content quality, personal projects |

**Numerical model:**
- Range: 0–10,000 (fine granularity allows nuanced growth tracking)
- Grows through training (affected by base attribute multiplier), gigs, and part-time jobs
- **Training does NOT decrease skills** — it costs money and adds stress (following Stardom 1 model). Part-time jobs DO have skill trade-offs (increase some, decrease others).
- Different training tiers (beginner/intermediate/advanced/expert) with increasing cost, effect, and stress

### 3.4 Inner Traits (2) — Double-Edged Swords

These are NOT "higher is better" stats. They create strategic tension — the player must decide the optimal range for each artist's career path.

| Trait | Key | Range | Positive Effects (moderate-high) | Negative Effects (excessive) |
|-------|-----|-------|--------------------------------|------------------------------|
| Confidence | CON | 0–100 | Big stage performance bonus, audition competitiveness, leadership in group activities | Conflicts with other artists, difficult to manage, diva behavior events |
| Rebellion | REB | 0–100 | Rock/hip-hop/alternative genre bonus, topic-generating charisma, unique persona | Skipping work, image controversies, PR crisis trigger, contract disputes |

**Gameplay implications:**
- A rock artist NEEDS moderate-high rebellion for genre authenticity bonuses
- But push it too far and they start missing gigs and generating scandals
- A variety show host benefits from high confidence for stage presence
- But excessive confidence makes them clash with co-stars and refuse direction
- The player must find the sweet spot for each artist's career path

**Numerical model:**
- Range: 0–100
- Changes through events, gig types, training, and player decisions
- Key thresholds trigger specific events (e.g., Rebellion > 70 → skip work chance, Confidence > 80 → refuse certain gigs)

### 3.5 Personality Spectrums (4) — Bipolar Axes

Inspired by MBTI's dimensional approach but NOT using MBTI labels. These are true spectrums where being on one side necessarily means being less of the other. They drift over time based on life events and player decisions.

| Spectrum | Left Pole (-100) | Right Pole (+100) | Gameplay Effect |
|----------|-----------------|-------------------|-----------------|
| Social | Introvert | Extrovert | Introvert: bonus to deep interviews, art films, solo creative work. Extrovert: bonus to variety shows, live content, fan events |
| Thinking | Intuitive | Logical | Intuitive: bonus to improvisation, emotional performances, artistic creation. Logical: bonus to business decisions, strategic planning, analytical roles |
| Action | Cautious | Adventurous | Cautious: stable growth, fewer negative events, but lower ceiling. Adventurous: high-risk-high-reward gigs available, but higher chance of incidents |
| Stance | Easygoing | Competitive | Easygoing: better team chemistry, group activities bonus. Competitive: better in competitions, award ceremonies, audition scenarios |

**Numerical model:**
- Range: -100 to +100
- Initial value set at artist creation
- Drifts based on events experienced (e.g., surviving a scandal might push Cautious→Adventurous)
- No "correct" position — different career paths favor different positions
- Can generate in-game "personality profile" as a topic/news event (nod to MBTI cultural trend)

**Design intent:**
- These are NOT directly trainable — the player influences them indirectly through career choices and event decisions
- Creates emergent personality: an artist who goes through many crises naturally becomes more adventurous and competitive
- Encourages the player to consider "who is this artist becoming" rather than just optimizing numbers

### 3.6 Image Tags (6) — Public Persona

Independent values that can coexist. An artist can be simultaneously Pure AND Sexy (the "innocent allure" archetype) or Cool AND Funny (the "unexpectedly hilarious cool person"). This is NOT a spectrum — each tag is its own dimension.

| Tag | Key | Description | Boosted By |
|-----|-----|-------------|-----------|
| Pure | PUR | Clean, approachable, wholesome | Costume choices, wholesome gigs, charitable events |
| Sexy | SEX | Physical allure, mature attractiveness | Costume choices, photobooks, certain performances |
| Cool | COO | Stylish, edgy, effortlessly fashionable | Fashion gigs, music genre choices, public attitude |
| Intellectual | ITE | Cultured, thoughtful, sophisticated | Cultural gigs, interview performance, creative output |
| Funny | FUN | Humorous, entertaining, variety-show energy | Variety show performance, public interactions, comedy roles |
| Mysterious | MYS | Enigmatic, selective, low-profile allure | Limited public appearances, selective gig strategy, event handling |

**Numerical model:**
- Range: 0–100 each, independent
- Changed by: costume/outfit selection (direct), gig types (cumulative), event decisions, aging (natural drift)
- Natural aging effect: Pure tends to decrease with age, other tags may shift
- Some tags are easier to maintain than others based on artist's base attributes and personality

**Gameplay implications:**
- Each gig/endorsement has "ideal image" requirements (e.g., a children's brand endorsement wants Pure ≥ 60)
- Image conflicts reduce success rate (high Sexy artist doing a children's show feels wrong)
- Image contrast can generate buzz (a normally Mysterious artist doing a comedy show = news topic → Popularity spike)
- Costumes are the most direct lever — changing outfits before a gig can shift image tags temporarily
- Long-term image is built through cumulative career choices, not just what you wear today

### 3.7 Auxiliary Stats (4) — Market Status Dashboard

These are system-level stats that reflect the artist's current market position. They are NOT directly trainable — they emerge from gameplay actions.

#### Recognition (知名度)
- **Range:** 0 → ∞ (uncapped, accumulative)
- **Direction:** Only increases, never decreases
- **Sources:** Completing gigs, scandals (yes, bad press is still press), awards, media exposure
- **Function:** Gig eligibility threshold — you need to be known enough to be considered for big opportunities
- **Design note:** Separating "how known" from "how liked" is critical. A scandal makes you MORE known, not less.
- **Normalization for gig thresholds:** Gig requirements use tiered Recognition brackets rather than raw values:

| Tier | Recognition Range | Label | Typical Gigs |
|------|------------------|-------|-------------|
| 0 | 0–99 | Unknown | Part-time jobs only |
| 1 | 100–499 | Newcomer | Minor gigs, local ads |
| 2 | 500–1,999 | Rising | Standard TV/film/music gigs |
| 3 | 2,000–4,999 | Established | Major gigs, endorsements |
| 4 | 5,000–14,999 | Star | Premium gigs, concert tours |
| 5 | 15,000+ | Superstar | Legendary gigs, international |

- Tier thresholds are data-configurable. Gig definitions reference tiers (e.g., `required_recognition_tier: 3`), not raw values.

#### Reputation (風評)
- **Range:** -100 ↔ +100 (single-axis spectrum)
- **Direction:** Shifts based on actions and events
- **Positive sources:** Quality work, charity, good PR crisis management
- **Negative sources:** Scandals, bad PR handling, controversial behavior
- **Function:** Determines WHICH opportunities are available — both extremes unlock unique content

| Reputation Range | Status | Available Opportunities |
|-----------------|--------|------------------------|
| +70 and above | National idol | Premium endorsements, goodwill ambassador, award ceremony host |
| +30 to +70 | Well-liked | Mainstream gigs, broad range of work |
| -30 to +30 | Neutral | Standard gigs, no special bonuses or penalties |
| -70 to -30 | Controversial | Talk show hot-seat, villain casting, gossip show features |
| -70 and below | Infamous | Underground events, "bad boy/girl" brand deals, BUT mainstream brands blacklist |

**Key design principle:** Both extremes of reputation are viable career strategies. "Going dark" (intentionally tanking reputation) is a legitimate high-risk-high-reward playstyle. This makes the PR crisis system more meaningful — a crisis isn't just "damage to fix" but a strategic crossroads.

#### Popularity (人氣)
- **Range:** 0–100
- **Direction:** Naturally decays over time; must be actively maintained
- **Sources:** Active gigs, media appearances, fan events, trending topics
- **Decay rate:** Increases if the artist has no public activity for extended periods
- **Function:** Determines gig pay rates (how "hot" you are right now affects your asking price), fan event turnout, album/film sales multiplier
- **Design note:** This captures the "15 minutes of fame" phenomenon. An artist can have high Recognition (everyone knows them) but low Popularity (nobody cares about them right now) — the classic "washed up" state. Recovering from low Popularity with high Recognition is a distinct gameplay challenge.

#### Stress (壓力)
- **Range:** 0–100
- **Direction:** Increases from work, training, negative events; decreases from rest and positive events
- **Threshold effects:**
  - 0–30: Healthy. No penalties.
  - 31–60: Fatigued. Training efficiency decreases. Minor mood events.
  - 61–80: Strained. Gig failure chance increases. Negative event trigger rate up.
  - 81–100: Breaking point. Risk of artist quitting, major scandals, health events.
- **Management:** Rest days, vacations, positive social events, certain personality types (Easygoing) recover faster
- **Design note:** Stress management is the core pacing mechanic. It prevents the player from simply grinding training non-stop and forces strategic scheduling decisions.

---

## 4. Core Gameplay Systems (v1)

### 4.1 Time & Scheduling System

- Base time unit: **1 week**
- Each week the player assigns activities for each artist (training, part-time job, gig, rest)
- Gig availability rotates on a **bi-weekly** cycle (different categories alternate)
- Weekdays: main scheduled activity
- Weekends: special activities (fan meetings, concerts, outings)
- Calendar-based events: award ceremonies at fixed dates, seasonal events

### 4.2 Training System

- Multiple training types, each increasing 1–2 professional skills
- Training tiers (beginner → expert) with increasing cost, effect, and stress
- Training does NOT decrease other skills (unlike part-time jobs), but costs money and adds stress
- Training efficiency is multiplied by relevant base attributes
- All training definitions are data-driven (RON/TOML)

### 4.3 Part-Time Job System

- Available from the start, used to build initial fame and earn small income
- Each job increases some skills/stats but decreases others (trade-off mechanic)
- Higher-paying jobs have higher stress costs and attribute trade-offs
- Some jobs have fame/stat prerequisites to unlock
- All job definitions are data-driven

### 4.4 Gig/Engagement System

Career-defining work opportunities across multiple categories:

| Category | Examples | Primary Skill |
|----------|----------|--------------|
| Music | Albums, singles, live concerts | Vocal |
| Film/TV | Movies, TV dramas, short dramas | Acting |
| Modeling | Fashion shows, photobooks, brand campaigns | Poise |
| Variety | Talk shows, game shows, specials | Eloquence |
| Endorsements | Brand deals, commercials | Image Tags match |
| Creative | Self-produced content, personal projects | Creativity |

- Each gig has requirements (minimum skill levels, Recognition threshold, Image tag preferences)
- Gig success depends on skill levels, personality fit, and current Popularity
- Compensation scales with Popularity
- Gig availability rotates bi-weekly and is data-driven
- Completing gigs affects: skills, Recognition, Reputation, Popularity, Stress, and Image Tags

### 4.5 Award System

Three major awards per year (dates and criteria are data-driven and extensible):

| Award | Timing | Core Criteria |
|-------|--------|--------------|
| Model Award | ~September | Poise + Image Tags (style-dependent) |
| Music Award | ~November | Vocal + Creativity + album performance |
| Film Award | ~December | Acting + Empathy + film performance |

- Artists must meet nomination criteria (e.g., released an album that charted)
- Final judging compares the artist's relevant stats against AI competitors
- Winning awards significantly boosts Recognition and Reputation
- Default win condition: win all 3 major awards within 3 years

### 4.6 Financial System

- Company has a single money pool
- Income: gig pay, album/film revenue, endorsement fees
- Expenses: training costs, event costs (concerts, photobooks), company upkeep, artist salaries
- Optional: stock market investment (higher risk/reward)
- **Bankruptcy model** (see Appendix A.13 for details): If company balance is negative for 4 consecutive weeks with no recovery path, game over triggers. Office can be downgraded (sold) to recover partial cost as an emergency measure.
- Financial milestones unlock company upgrades

### 4.7 Artist Recruitment System

- Potential artists appear at specific locations on specific days
- Dialogue choices during first meeting affect signing conditions (commission rate)
- Failed negotiations (twice with same artist) may permanently lock them out
- Each artist has unique base attributes, personality spectrums, and growth potential
- All artist definitions are data-driven

---

## 5. v1 Special Feature Systems

These are the four additional systems selected from the Stardom 3 era for inclusion in the initial version.

### 5.1 PR Crisis System

Random events that threaten (or provide opportunity for) an artist's career:

- **Event types:** Scandals, stalker incidents, threatening letters, rumors, paparazzi exposés
- **Player response options:** Each crisis presents 2–3 choices with different outcomes
- **Outcomes affect:** Reputation (positive or negative), Popularity (controversy generates buzz), Stress, and potentially Image Tags
- **Strategic depth:** A crisis is not just "damage to fix" — depending on the artist's current career strategy (mainstream vs. edgy), the "right" response may differ
- **PR crisis events are data-driven:** modders can add custom crisis scenarios

### 5.2 Office Upgrade System

Visual and functional progression of the talent agency:

- Company starts in a small, modest office
- As revenue and reputation milestones are hit, upgrade options become available
- Each upgrade tier provides: visual change (new office background), functional bonuses (more artist slots, better training facilities, reduced costs)
- Upgrades cost significant money — a strategic investment decision
- Office state is visible in the main management screen, providing tangible growth feedback

### 5.3 Costume/Outfit System

Clothing affects Image Tags and provides visual customization:

- Each artist has an outfit slot
- Outfits have Image Tag modifiers (e.g., "Elegant Dress: Pure +15, Intellectual +10" or "Leather Jacket: Cool +20, Rebellion trait +5")
- Outfits can be purchased from shops or obtained through events
- Outfit choice before a gig applies a **temporary** Image Tag modifier on top of the artist's cumulative image values (see Appendix A.8)
- Cumulative image changes slowly through gig types and events (±1–5 per gig)
- All outfits are data-driven (easy to add via mods)

### 5.4 Mini-Game System

Interactive mini-games that break up the management loop:

- Triggered during variety show gigs, training sessions, or special events
- Performance in mini-games affects gig success ratings and stat gains
- Mini-game types (data-driven, extensible):
  - **Rhythm game** (for music/dance gigs)
  - **Quick-time reaction** (for variety shows)
  - **Memory match** (for training)
  - **Trivia/quiz** (for intellectual variety shows)
- Mini-games are optional — player can choose to auto-resolve with a skill-based check instead (accessibility). Auto-resolve formula: `success_score = relevant_skill * 0.7 + random(0, relevant_skill * 0.3)`. If `success_score >= gig_difficulty_threshold`, the gig succeeds with a "standard" rating; exceeding by 20%+ yields "excellent".

---

## 6. Relationship System (Framework)

v1 builds the underlying framework; romance content is added as data modules later.

### 6.1 Core Mechanics

- Each NPC has an **Affinity** value toward the player (and toward each artist)
- **Affinity range:** 0–100 (starts at a data-defined default, typically 10–30)
- **Relationship type** is a separate enum tag (Friendship / Rivalry / Romance / Mentorship), determined by events and player choices, NOT by affinity value alone
- Affinity changes through: dialogue choices, gifts, events, work interactions
- High affinity unlocks: special events, exclusive gigs, story branches

### 6.2 Future Expansion

- Romance storylines as data modules
- NPC-specific event chains
- Artist-to-artist relationships (chemistry, rivalries)

---

## 7. Narrative System (Framework)

v1 builds a complete scripting engine; story content is incrementally added as data modules.

### 7.1 Architecture

- Event/dialogue scripting engine built into Game Core
- Scripts defined in external data files
- Support for: branching dialogue, conditional triggers, variable tracking, cutscene sequencing
- Event triggers can be: date-based, stat-threshold-based, relationship-based, random

### 7.2 v1 Content Scope

- Basic opening scenario and tutorial
- Award ceremony scenes
- Ending sequences (multiple endings, data-driven)
- PR crisis dialogue trees
- Minimal artist personal storylines (framework demonstration)

### 7.3 Design Philosophy

> "Rich, well-founded gameplay framework is what retains players. Story is what makes the experience memorable."

The narrative engine must be robust enough to support deep storytelling, but v1 prioritizes gameplay system completeness over narrative content volume.

---

## 8. Post-v1 Expansion Roadmap

Features designed into the architecture but not implemented in v1:

| Feature | Description | Architecture Impact |
|---------|-------------|-------------------|
| Artist mode | Player IS the artist | Presentation layer change; Game Core shared |
| Group/band system | Form artist groups, build chemistry | Entity composition in Game Core |
| Self-production | Company produces own albums/shows | Gig system extension |
| International expansion | Overseas training, international gigs | Location system extension |
| Mod editor UI | Visual editor for content creation | Reads/writes same data files Game Core uses |
| Deep romance content | Full relationship storylines | Data modules for narrative system |
| Difficulty modes | Easy/normal/hard settings | Parameter multipliers in Game Core |
| Event tracker/journal | In-game notebook for tracking event chains | UI feature + event system metadata |

---

## 9. Data Architecture Summary

All game content is defined in external data files to support modding and extensibility:

| Content Type | File Format | Examples |
|-------------|-------------|---------|
| Artist definitions | RON/TOML | Base stats, personality, growth curves, backstory |
| Gig definitions | RON/TOML | Requirements, rewards, duration, category |
| Training definitions | RON/TOML | Skill effects, cost, stress, tier progression |
| Part-time job definitions | RON/TOML | Pay, stat changes, prerequisites |
| Event/story scripts | RON/custom | Trigger conditions, dialogue trees, outcomes |
| Award definitions | RON/TOML | Timing, criteria, competitor profiles |
| Outfit definitions | RON/TOML | Image tag modifiers, cost, visual asset reference |
| PR crisis scenarios | RON/TOML | Trigger conditions, choices, outcomes |
| Office upgrade tiers | RON/TOML | Cost, bonuses, visual asset reference |
| Mini-game definitions | RON/TOML | Type, difficulty, reward formulas |

---

## 10. Reference Research

Detailed game mechanics research for the original Stardom series is available in:

- `research/stardom1-game-mechanics-research.md` — Stardom 1 (1995) complete system analysis
- `research/stardom2-game-mechanics.md` — Stardom 2 (1998) complete system analysis
- Stardom 3 and series-wide research was conducted and key findings are integrated into this design

### Key sources:
- 巴哈姆特 Stardom forum archives
- Reko Wiki — 明星志願系列
- 幻光星宇 classic game archives
- Steam community guides
- Various fan blogs and wikis (see research files for full source lists)

---

## Appendix A: Numerical Models & Formulas

This appendix provides the quantitative models referenced by the systems above. All constants are data-configurable and represent recommended defaults.

### A.1 Training Efficiency Formula

Training gain per session:

```
effective_gain = base_gain_for_tier
                 * (1.0 + base_attribute_bonus)
                 * condition_modifier

base_attribute_bonus:
  primary_attribute:   (attribute_value - 50) / 100   (range: -0.49 to +0.50)
  secondary_attribute: (attribute_value - 50) / 200   (range: -0.245 to +0.25)
  total = primary + secondary (if applicable)

condition_modifier:
  stress 0–30:   1.0
  stress 31–60:  0.85
  stress 61–80:  0.65
  stress 81–100: 0.40
```

Example: Dance training (intermediate, base_gain=80), artist has STA=70 (primary), stress=25:
- `base_attribute_bonus = (70-50)/100 = 0.20`
- `effective_gain = 80 * 1.20 * 1.0 = 96`

### A.2 Training Tiers (Default Values)

| Tier | Cost ($) | Base Skill Gain | Stress Increase | Unlock Condition |
|------|----------|----------------|-----------------|-----------------|
| Beginner | 8,000 | 40 | +5 | Always available |
| Intermediate | 16,000 | 80 | +10 | Skill ≥ 1,000 |
| Advanced | 28,000 | 130 | +16 | Skill ≥ 3,000 |
| Expert | 44,000 | 180 | +22 | Skill ≥ 6,000 |

### A.3 Popularity Decay Model

```
weekly_decay = base_decay + inactivity_penalty

base_decay:         -2 per week (always applies)
inactivity_penalty: -0 if artist had public activity this week
                    -2 if 1 week inactive
                    -4 if 2 consecutive weeks inactive
                    -6 if 3+ consecutive weeks inactive (caps here)

Minimum Popularity: 0
```

An artist with zero activity loses: week 1 = -4, week 2 = -6, week 3+ = -8/week. Active artists only lose -2/week, easily offset by gigs.

### A.4 Stress Threshold Effects

| Stress Range | Training Efficiency | Gig Failure Chance | Event Trigger |
|-------------|--------------------|--------------------|--------------|
| 0–30 | 100% | 0% base | None |
| 31–60 | 85% | +5% per 10 stress above 30 | Minor mood events (data-driven) |
| 61–80 | 65% | +10% per 10 stress above 60 | Skip-work chance if Rebellion > 50 |
| 81–100 | 40% | +15% per 10 stress above 80 | Artist quit chance: (stress - 80) * 2% per week |

Example: Stress=90, Rebellion=60 → Gig failure: +15% (stress 81-90), skip-work: active, quit chance: 20%/week.

### A.5 Gig Success Calculation

```
success_score = weighted_skill_sum
                * personality_fit_modifier
                * image_match_modifier
                * popularity_modifier

weighted_skill_sum:
  Each gig defines required skills with weights (e.g., Vocal: 0.6, Dance: 0.3, Poise: 0.1)
  Sum = Σ(skill_value * weight)

personality_fit_modifier:
  Each gig may prefer a personality range (e.g., "prefers Extrovert > 30")
  Match: 1.0–1.15 bonus. Mismatch: 0.85–1.0 penalty. Neutral: 1.0.

image_match_modifier:
  Each gig has optional ideal_image_tags (e.g., Pure ≥ 40, Cool ≥ 30)
  Per tag: if met, +0.05 bonus. If strongly exceeded (2x threshold), +0.10.
  If a "conflicting" tag is high (defined per gig), -0.10 penalty.
  Total modifier: clamped to 0.80–1.20.

popularity_modifier:
  1.0 + (popularity - 50) / 200
  Range: 0.75 (pop=0) to 1.25 (pop=100)
```

### A.6 Inner Trait Threshold Events (Defaults)

| Trait | Threshold | Event |
|-------|-----------|-------|
| Rebellion > 50 | Stress ≥ 61 | Skip-work chance: (rebellion - 50) * 1.5% |
| Rebellion > 70 | Any | PR crisis trigger rate +20% |
| Rebellion > 85 | Any | Contract dispute event possible |
| Confidence > 60 | Failed audition | Mood penalty doubled |
| Confidence > 75 | Any | May refuse gigs below Recognition tier 3 |
| Confidence > 90 | Group activity | Clash event: chemistry penalty with Easygoing artists |

All thresholds and events are data-driven — these are recommended defaults.

### A.7 Personality Spectrum Gameplay Modifiers

Each spectrum position provides a modifier to relevant activities. The modifier scales linearly with distance from center (0):

```
modifier = |spectrum_value| / 100 * max_bonus

max_bonus per spectrum: 0.15 (15%)
```

| Spectrum | Left Pole Bonus | Right Pole Bonus |
|----------|----------------|-----------------|
| Social (-100 to +100) | Introvert: solo creative gigs, deep interviews, art film roles | Extrovert: variety shows, fan events, live content, group chemistry |
| Thinking | Intuitive: improvisation gigs, emotional roles, songwriting | Logical: business events, analytical roles, strategic PR decisions |
| Action | Cautious: -20% negative event chance, -10% max gig reward | Adventurous: +20% negative event chance, +10% max gig reward |
| Stance | Easygoing: +15% group chemistry, -10% competition score | Competitive: -10% group chemistry, +15% competition/audition score |

### A.8 Image Tag Dynamics

**Outfit (temporary) vs. Cumulative (permanent):**
- Outfit modifiers apply ON TOP of cumulative values for the duration of a gig
- Cumulative values change by ±1–5 per gig/event (slow drift)
- Outfit modifier range: typically ±10–25 per tag

**Age effect on Image Tags:**
```
If artist_age > 25:
  Pure -= (artist_age - 25) * 0.3 per year (natural decay, floor 0)
```
Other tags are not affected by age by default (data-configurable).

### A.9 Contract & Recruitment Model

**Commission rate:** The agency's cut of gig income.
- Base rate: 30% (data-configurable per artist)
- Dialogue choices during recruitment: ±5–10% adjustment
- Range: 15%–50%

**Contract terms:**
- Duration: 1 year, auto-renew unless artist Affinity < 20 or triggered by event
- No minimum gig quota in v1 (simplification from Stardom 2)
- Artist may request renegotiation if their Recognition tier increases by 2+

**Failed recruitment:**
- First failed negotiation: can retry next week at same location
- Second consecutive failure with same artist: locked out for 6 months (data-configurable)

### A.10 Scheduling & Gig Duration

- Base unit: 1 week
- Each artist has 1 activity slot per week
- Gigs have a **duration** measured in weeks (1–8 typical range)
- During a multi-week gig, the artist is locked into that activity (cannot train, work part-time, or accept other gigs)
- Short gigs (1 week): ads, variety show appearances, photoshoots
- Medium gigs (2–4 weeks): TV drama episodes, album recording
- Long gigs (4–8 weeks): feature films, concert tour preparation
- Bi-weekly rotation applies to **available new gigs**, not ongoing ones

### A.11 Data File Format Strategy

| Content Type | Format | Rationale |
|-------------|--------|-----------|
| Structured game data (artists, gigs, items, awards) | **RON** | Rust-native, type-safe, good enum support |
| Configuration & settings | **TOML** | Human-friendly for player/modder tweaking |
| Event/dialogue scripts | **RON with embedded script blocks** | Keep one ecosystem; script blocks use a simple DSL parsed by the narrative engine |

The custom DSL for narrative scripts will be documented separately when the narrative engine is designed.

### A.12 Game Core ↔ Presentation Layer Interface (Sketch)

```rust
// Commands: Presentation → Game Core
enum GameCommand {
    AdvanceWeek,
    AssignActivity { artist_id: ArtistId, activity: Activity },
    SignArtist { artist_id: ArtistId, commission: f32 },
    PurchaseOutfit { artist_id: ArtistId, outfit_id: OutfitId },
    RespondToCrisis { crisis_id: CrisisId, choice: usize },
    UpgradeOffice { tier: OfficeTier },
    // ... extensible
}

// State: Game Core → Presentation
struct GameState {
    calendar: Calendar,          // current date, week, year
    company: CompanyState,       // money, office tier, reputation
    artists: Vec<ArtistState>,   // all signed artists with full attribute state
    available_gigs: Vec<Gig>,    // current rotation of available gigs
    active_events: Vec<Event>,   // ongoing events/crises requiring response
    // ... extensible
}

// Presentation reads GameState each frame, renders UI accordingly.
// Presentation sends GameCommands based on player input.
// Game Core processes commands, updates GameState, returns results.
```

This is a conceptual sketch — actual API will be designed during implementation planning.

### A.13 Bankruptcy Model

**Company balance can go negative** (debt is allowed). This prevents instant game-over from a single expensive purchase.

**Bankruptcy trigger conditions:**
```
IF company_balance < 0
   AND consecutive_negative_weeks >= 4
   AND no_pending_gig_income (no artist has a gig completing within 2 weeks)
THEN trigger bankruptcy → game over
```

**Emergency measures available to the player before bankruptcy:**
- **Downgrade office:** Sell current office tier for 40% of its upgrade cost, reverting to previous tier
- **Terminate artist contract:** Save on salary expenses (but lose the artist permanently)
- **Take a loan:** If available from in-game bank (interest accrues weekly)

**What counts as "balance":**
- Current cash only. Unrealized assets (office value, contract potential) are NOT counted — the player must actively liquidate them.
- Pending gig income (artist currently mid-gig with known payout) counts as a recovery path and pauses the 4-week bankruptcy timer.

**Week-by-week process:**
1. Week ends → calculate income - expenses → update balance
2. If balance < 0: increment `consecutive_negative_weeks` counter
3. If balance ≥ 0: reset counter to 0
4. If counter reaches 4 and no pending income: bankruptcy event triggers
5. Bankruptcy event shows a narrative scene, then game over (with option to reload save)
