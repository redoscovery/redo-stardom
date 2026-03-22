use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use stardom_core::company::OfficeTier;
use stardom_core::game::GameCommand;
use stardom_core::office;

use crate::data_loading::GameCatalogs;
use crate::game_bridge::GameWorld;

use super::SelectedArtist;
use super::display::{activity_text, office_tier_text, recognition_tier_text};
use super::week_plan::{PlannedActivity, WeekPlan};
use super::week_report::WeekReport;

#[derive(Resource, Default)]
pub struct ActiveTab(usize); // 0=artist, 1=gigs, 2=shop, 3=recruit

pub fn central_tabs_ui(
    mut contexts: EguiContexts,
    mut game: ResMut<GameWorld>,
    selected: Res<SelectedArtist>,
    catalogs: Option<Res<GameCatalogs>>,
    mut week_plan: ResMut<WeekPlan>,
    mut active_tab: ResMut<ActiveTab>,
    report: Option<Res<WeekReport>>,
) {
    // Skip when WeekReport modal is active
    if report.is_some() {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let tab_labels = ["藝人資料", "通告市場", "商店", "招募"];

    egui::CentralPanel::default().show(ctx, |ui| {
        // Tab bar
        ui.horizontal(|ui| {
            for (i, label) in tab_labels.iter().enumerate() {
                if ui.selectable_label(active_tab.0 == i, *label).clicked() {
                    active_tab.0 = i;
                }
            }
        });
        ui.separator();

        match active_tab.0 {
            0 => tab_artist(ui, &game, &selected, &catalogs, &mut week_plan),
            1 => tab_gigs(ui, &game, &selected, &mut week_plan),
            2 => tab_shop(ui, &mut game, &selected),
            3 => tab_recruit(ui, &mut game),
            _ => {}
        }
    });
}

// ── Tab 0: Artist Detail ──

fn tab_artist(
    ui: &mut egui::Ui,
    game: &GameWorld,
    selected: &SelectedArtist,
    catalogs: &Option<Res<GameCatalogs>>,
    week_plan: &mut WeekPlan,
) {
    let Some(idx) = selected.0 else {
        ui.label("請從左側藝人名冊選擇一位藝人。");
        return;
    };
    let Some(artist) = game.0.artists.get(idx).cloned() else {
        ui.label("選擇無效。");
        return;
    };

    let training_defs = catalogs
        .as_ref()
        .map(|c| c.training.clone())
        .unwrap_or_default();
    let job_defs = catalogs
        .as_ref()
        .map(|c| c.jobs.clone())
        .unwrap_or_default();
    let current_plan_label = week_plan.get(idx).map(|a| a.label());

    let mut new_plan: Option<PlannedActivity> = None;
    let mut cancel_plan = false;

    ui.heading(&artist.name);
    ui.label(format!(
        "年齡：{} | 活動：{}",
        artist.age,
        activity_text(&artist.current_activity)
    ));
    ui.separator();

    // Activity assignment at top
    ui.heading("安排活動");
    if artist.is_locked() {
        ui.label(format!("[鎖定] 通告中（剩餘 {} 週）", artist.locked_weeks));
    } else if let Some(label) = &current_plan_label {
        ui.horizontal(|ui| {
            ui.label(format!("[已安排] {}", label));
            if ui.button("取消").clicked() {
                cancel_plan = true;
            }
        });
    } else {
        ui.label("-- 訓練 --");
        for training in &training_defs {
            let cost = training.tiers.first().map(|t| t.cost).unwrap_or(0);
            if ui
                .button(format!("{} (${})", training.name, cost))
                .clicked()
            {
                new_plan = Some(PlannedActivity::Training(training.clone()));
            }
        }
        ui.label("-- 打工 --");
        for job in &job_defs {
            if ui.button(format!("{} (+${})", job.name, job.pay)).clicked() {
                new_plan = Some(PlannedActivity::Job(job.clone()));
            }
        }
        ui.label("-- 其他 --");
        if ui.button("休息").clicked() {
            new_plan = Some(PlannedActivity::Rest);
        }
    }
    ui.separator();

    // All stats in single column, scrollable
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("基礎屬性");
        ui.label(format!(
            "體能：{}  智識：{}  共情：{}  魅力：{}",
            artist.base_attributes.stamina,
            artist.base_attributes.intellect,
            artist.base_attributes.empathy,
            artist.base_attributes.charm
        ));

        ui.add_space(4.0);
        ui.heading("專業技能");
        let skills = &artist.skills;
        for (name, val) in [
            ("歌藝", skills.vocal),
            ("演技", skills.acting),
            ("舞藝", skills.dance),
            ("儀態", skills.poise),
            ("口才", skills.eloquence),
            ("創作", skills.creativity),
        ] {
            ui.horizontal(|ui| {
                ui.label(format!("{name}：{val:>5}"));
                ui.add(egui::ProgressBar::new(val as f32 / 10000.0).desired_width(200.0));
            });
        }

        ui.add_space(4.0);
        ui.heading("內在特質");
        ui.label(format!(
            "自信：{}  叛逆：{}",
            artist.traits.confidence, artist.traits.rebellion
        ));

        ui.add_space(4.0);
        ui.heading("性格光譜");
        ui.label(format!(
            "社交：{} ({})  思維：{} ({})",
            artist.personality.social,
            if artist.personality.social < 0 {
                "內斂"
            } else {
                "外放"
            },
            artist.personality.thinking,
            if artist.personality.thinking < 0 {
                "直覺"
            } else {
                "邏輯"
            },
        ));
        ui.label(format!(
            "行事：{} ({})  處世：{} ({})",
            artist.personality.action,
            if artist.personality.action < 0 {
                "謹慎"
            } else {
                "冒險"
            },
            artist.personality.stance,
            if artist.personality.stance < 0 {
                "隨和"
            } else {
                "好勝"
            },
        ));

        ui.add_space(4.0);
        ui.heading("形象標籤");
        let img = &artist.image;
        ui.label(format!(
            "清純：{}  性感：{}  酷帥：{}  知性：{}  搞笑：{}  神秘：{}",
            img.pure, img.sexy, img.cool, img.intellectual, img.funny, img.mysterious
        ));

        ui.add_space(4.0);
        ui.heading("市場狀態");
        ui.label(format!(
            "知名度：{} ({})  風評：{}  人氣：{}  壓力：{}",
            artist.stats.recognition,
            recognition_tier_text(artist.stats.recognition_tier()),
            artist.stats.reputation,
            artist.stats.popularity,
            artist.stats.stress,
        ));
    });

    // Apply plan changes
    if cancel_plan {
        week_plan.cancel(idx);
    } else if let Some(activity) = new_plan {
        week_plan.assign(idx, activity);
    }
}

// ── Tab 1: Gig Market ──

fn tab_gigs(
    ui: &mut egui::Ui,
    game: &GameWorld,
    selected: &SelectedArtist,
    week_plan: &mut WeekPlan,
) {
    ui.heading("可接通告");
    let gig_catalog = &game.0.gig_catalog;
    if gig_catalog.is_empty() {
        ui.label("目前沒有可接的通告。");
        return;
    }
    egui::ScrollArea::vertical().show(ui, |ui| {
        for gig in gig_catalog {
            ui.group(|ui| {
                ui.label(format!("{} ({:?})", gig.name, gig.category));
                ui.label(format!(
                    "工期：{} 週 | 片酬：${} | 門檻：{} | 壓力：+{}",
                    gig.duration_weeks,
                    gig.base_pay,
                    recognition_tier_text(gig.required_recognition_tier),
                    gig.stress_cost
                ));
                if let Some(idx) = selected.0 {
                    if ui.button("接下通告").clicked() {
                        week_plan.assign(idx, PlannedActivity::Gig(gig.clone()));
                    }
                } else {
                    ui.label("(請先選擇一位藝人)");
                }
            });
        }
    });
}

// ── Tab 2: Shop (Office + Outfits) ──

fn tab_shop(ui: &mut egui::Ui, game: &mut GameWorld, selected: &SelectedArtist) {
    let office_tier = game.0.company.office_tier;
    let balance = game.0.company.balance;
    let office_upgrades = game.0.office_upgrades.clone();
    let outfit_catalog = game.0.outfit_catalog.clone();
    let owned_outfits = game.0.owned_outfits.clone();

    let mut pending_cmd: Option<GameCommand> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Office
        ui.heading("辦公室");
        ui.label(format!("目前等級：{}", office_tier_text(office_tier)));
        if let Some(next) = office::next_upgrade(office_tier, &office_upgrades) {
            ui.label(format!(
                "升級至{}：${}  (+{} 藝人名額，{}% 訓練折扣)",
                office_tier_text(next.tier),
                next.cost.0,
                next.max_artists_bonus,
                next.training_cost_discount_pct
            ));
            if office::can_afford(balance, next) {
                if ui.button("升級辦公室").clicked() {
                    pending_cmd = Some(GameCommand::UpgradeOffice);
                }
            } else {
                ui.label("資金不足");
            }
        } else {
            ui.label("已達最高等級！");
        }
        if office_tier > OfficeTier::Starter {
            let refund = office::downgrade_refund(office_tier, &office_upgrades);
            if ui
                .button(format!("降級辦公室（退款 ${}）", refund.0))
                .clicked()
            {
                pending_cmd = Some(GameCommand::DowngradeOffice);
            }
        }

        ui.separator();

        // Outfits
        ui.heading("服裝");
        if outfit_catalog.is_empty() {
            ui.label("商店沒有服裝。");
        }
        for outfit in &outfit_catalog {
            ui.group(|ui| {
                ui.label(&outfit.name);
                ui.label(format!("價格：${}", outfit.cost.0));
                let modifiers: Vec<String> = outfit
                    .image_modifiers
                    .iter()
                    .map(|(tag, val)| format!("{:?} {:+}", tag, val))
                    .collect();
                if !modifiers.is_empty() {
                    ui.label(modifiers.join(", "));
                }
                let owned = owned_outfits.contains(&outfit.id);
                if owned {
                    ui.label("已擁有");
                    if let Some(idx) = selected.0
                        && ui.button("穿戴").clicked()
                    {
                        pending_cmd = Some(GameCommand::EquipOutfit {
                            artist_index: idx,
                            outfit_id: outfit.id,
                        });
                    }
                } else if ui.button("購買").clicked() {
                    pending_cmd = Some(GameCommand::PurchaseOutfit {
                        outfit_id: outfit.id,
                    });
                }
            });
        }
    });

    if let Some(cmd) = pending_cmd {
        game.command(cmd);
    }
}

// ── Tab 3: Recruitment ──

fn tab_recruit(ui: &mut egui::Ui, game: &mut GameWorld) {
    ui.heading("招募");
    let prospects = game.0.prospects.clone();
    let total_weeks = game.0.calendar.total_weeks_elapsed;
    let artist_count = game.0.artists.len();
    let max_artists = game.0.company.max_artists as usize;

    if prospects.is_empty() {
        ui.label("目前沒有可招募的藝人。");
        return;
    }

    let mut pending_cmd: Option<GameCommand> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (i, prospect) in prospects.iter().enumerate() {
            ui.group(|ui| {
                ui.label(format!(
                    "{} ({}歲)",
                    prospect.definition.name, prospect.definition.starting_age
                ));
                ui.label(format!(
                    "出沒地點：{} | 出現日：{}",
                    prospect.location, prospect.available_day
                ));
                ui.label(format!(
                    "抽成：{}%",
                    (prospect.base_commission * 100.0) as i32
                ));
                if prospect.is_locked(total_weeks) {
                    ui.label("[鎖定] 暫時無法接觸");
                } else if artist_count < max_artists {
                    if ui.button("簽約").clicked() {
                        pending_cmd = Some(GameCommand::SignArtist {
                            prospect_index: i,
                            commission_adjustment: 0,
                        });
                    }
                } else {
                    ui.label("名額已滿");
                }
            });
        }
    });

    if let Some(cmd) = pending_cmd {
        game.command(cmd);
    }
}
