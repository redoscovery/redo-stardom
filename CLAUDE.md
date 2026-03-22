# CLAUDE.md — REDÓ Stardom

## Project Overview

REDÓ Stardom 是一款明星經紀模擬遊戲，靈感來自經典 DOS 遊戲《明星志願》系列。
目前處於 v0.1.1 可玩原型階段（egui prototype UI）。

## Architecture

**Two-layer Cargo workspace:**

- `crates/stardom-core/` — 純 Rust library，零 Bevy 依賴。所有遊戲邏輯、數值模型、資料載入。可獨立 `cargo test`。
- `crates/redo-stardom/` — Bevy 0.18 app，負責 UI 渲染、輸入處理。依賴 stardom-core。

**Game Core 的 API 邊界：**
- Presentation 層透過 `GameCommand` enum 驅動所有遊戲邏輯
- Presentation 層從 `GameState` struct 讀取所有狀態
- 兩層之間不共用 Bevy 類型

## Tech Stack

- Rust 1.94, edition 2024
- Bevy 0.18 + bevy_egui 0.39
- serde + ron (遊戲資料) + toml (設定)
- rand 0.9
- Fusion Pixel Font 12px (OFL-1.1)

## Key Conventions

### Code

- 遊戲內容全部資料驅動：`data/` 目錄下的 RON 檔案定義所有藝人、訓練、打工、通告、獎項、危機、服裝
- 設定用 TOML：`config/settings.toml`
- 新增資料類型時在 `types.rs` 的 `id_newtype!` 巨集加 ID type
- 重複的 clamp 邏輯用 `attribute.rs` 的 `clamp_fields!` 巨集
- `ProfessionalSkills` 有 `get/get_mut/apply_gain/apply_loss` 方法，不要在其他地方重複 match SkillTarget

### UI (bevy_egui)

**重要：面板建立順序**
egui 的 TopBottomPanel/SidePanel 必須在 CentralPanel 之前建立。所有 InGame UI 系統用 `.chain()` 串接：
```
hud(top) → game_log(bottom) → dashboard(left) → central_tabs(central) → events → week_report
```

- 字型在 `Update` schedule 第一幀載入（不是 Startup，因 egui context 尚未就緒）
- UI 系統放在 `EguiPrimaryContextPass` schedule（不是 `Update`）
- `run_if` 條件放在系統函式內部（early return），不用 `.run_if()`，因為與 `.chain()` 衝突
- egui ctx 取得用 `let Ok(ctx) = contexts.ctx_mut() else { return; };`

### Game Loop

遊戲迴圈的流程（每週）：
1. 玩家為每位非鎖定藝人安排活動（寫入 `WeekPlan`，不立即生效）
2. 全部安排完後按「推進一週」
3. `execute_week()`: 快照狀態 → 逐一發送 GameCommand → AdvanceWeek → 比對前後差異 → 產生 WeekReport
4. 顯示結算報表，按「確定」關閉
5. 下一週開始

### Data Files

```
data/
├── artists/         # ArtistDefinition (sample) + ArtistProspect (recruitable)
├── awards/          # AwardDef (3 default awards)
├── crises/          # CrisisDef (3 default crises)
├── gigs/            # GigDef (3 default gigs)
├── jobs/            # JobDef (3 default jobs)
├── offices/         # OfficeUpgradeDef (3 tiers)
├── outfits/         # OutfitDef (4 default outfits)
├── scripts/         # ScriptDef (1 sample narrative)
└── training/        # TrainingDef (3 default trainings)
```

### Brand

- 系列：REDÓSCOVERY（全大寫正式）/ Redóscovery（一般文字）
- 遊戲：REDÓ Stardom
- 程式碼/URL：`redoscovery` / `redo-stardom`（無附加符號）
- ADR-001 記錄完整規範

## Testing

```bash
cargo test --workspace          # 137 tests
cargo clippy --workspace -- -D warnings
cargo fmt --check --all
cargo run -p redo-stardom       # 啟動遊戲
```

## Important Files

| 檔案 | 說明 |
|------|------|
| `crates/stardom-core/src/game.rs` | GameState, GameCommand, advance_week() — 遊戲核心 |
| `crates/stardom-core/src/types.rs` | 所有 ID newtypes, Money, Activity |
| `crates/redo-stardom/src/ui/mod.rs` | UI 系統註冊與排序 |
| `crates/redo-stardom/src/ui/central_tabs.rs` | 主要遊戲畫面（4 tab） |
| `crates/redo-stardom/src/ui/hud.rs` | HUD + 推進一週 + execute_week() |
| `crates/redo-stardom/src/ui/week_plan.rs` | 活動安排暫存（deferred execution） |
| `docs/superpowers/specs/` | 設計規格（early-stage，程式碼為準） |
| `config/settings.toml` | 可調整的遊戲設定 |

## Known Issues

- egui 佈局仍需改善：元件寬度應用 `ui.available_width()` 自適應
- Fusion Pixel 12px 繁中覆蓋率 92%，部分罕用字可能缺字
- `available_gigs` 的 bi-weekly rotation 只寫 log 未實際更新 UI 可見列表
