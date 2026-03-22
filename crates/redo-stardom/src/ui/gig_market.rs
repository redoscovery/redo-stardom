use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use stardom_core::company::OfficeTier;
use stardom_core::game::GameCommand;
use stardom_core::office;

use crate::game_bridge::GameWorld;
use crate::states::AppState;

use super::SelectedArtist;
use super::display::{office_tier_text, recognition_tier_text};

pub struct GigMarketPlugin;

impl Plugin for GigMarketPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            gig_market_ui.run_if(in_state(AppState::InGame)),
        );
    }
}

fn gig_market_ui(
    mut contexts: EguiContexts,
    mut game: ResMut<GameWorld>,
    selected: Res<SelectedArtist>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Snapshot data before entering egui closures to avoid borrow conflicts
    let gig_catalog = game.0.gig_catalog.clone();
    let office_tier = game.0.company.office_tier;
    let balance = game.0.company.balance;
    let office_upgrades = game.0.office_upgrades.clone();
    let outfit_catalog = game.0.outfit_catalog.clone();
    let owned_outfits = game.0.owned_outfits.clone();
    let prospects = game.0.prospects.clone();
    let total_weeks = game.0.calendar.total_weeks_elapsed;
    let artist_count = game.0.artists.len();
    let max_artists = game.0.company.max_artists as usize;

    let mut pending_cmd: Option<GameCommand> = None;

    egui::SidePanel::right("market")
        .min_width(280.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // --- Available Gigs ---
                egui::CollapsingHeader::new("可接通告")
                    .default_open(true)
                    .show(ui, |ui| {
                        if gig_catalog.is_empty() {
                            ui.label("目前沒有可接的通告。");
                        }
                        for gig in &gig_catalog {
                            ui.group(|ui| {
                                ui.label(format!("{} ({:?})", gig.name, gig.category));
                                ui.label(format!(
                                    "工期：{} 週 | 片酬：${}",
                                    gig.duration_weeks, gig.base_pay
                                ));
                                ui.label(format!(
                                    "門檻：{} | 壓力：+{}",
                                    recognition_tier_text(gig.required_recognition_tier),
                                    gig.stress_cost
                                ));
                                if let Some(idx) = selected.0 {
                                    if ui.button("接下通告").clicked() {
                                        pending_cmd = Some(GameCommand::AssignGig {
                                            artist_index: idx,
                                            gig: gig.clone(),
                                        });
                                    }
                                } else {
                                    ui.label("(請先選擇一位藝人)");
                                }
                            });
                        }
                    });

                ui.separator();

                // --- Recruitment ---
                egui::CollapsingHeader::new("招募")
                    .default_open(false)
                    .show(ui, |ui| {
                        if prospects.is_empty() {
                            ui.label("目前沒有可招募的藝人。");
                        }
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

                ui.separator();

                // --- Office Upgrades ---
                egui::CollapsingHeader::new("辦公室")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(format!("目前等級：{}", office_tier_text(office_tier)));
                        if let Some(next) = office::next_upgrade(office_tier, &office_upgrades) {
                            ui.label(format!(
                                "升級至{}：${}",
                                office_tier_text(next.tier),
                                next.cost.0
                            ));
                            ui.label(format!(
                                "  +{} 藝人名額，{}% 訓練折扣",
                                next.max_artists_bonus, next.training_cost_discount_pct
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
                    });

                ui.separator();

                // --- Outfit Shop ---
                egui::CollapsingHeader::new("服裝")
                    .default_open(false)
                    .show(ui, |ui| {
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
            });
        });

    // Apply any pending command after UI rendering is done
    if let Some(cmd) = pending_cmd {
        game.command(cmd);
    }
}
