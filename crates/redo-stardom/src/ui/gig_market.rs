use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use stardom_core::company::OfficeTier;
use stardom_core::game::GameCommand;
use stardom_core::office;

use crate::game_bridge::GameWorld;
use crate::states::AppState;

use super::SelectedArtist;

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
                egui::CollapsingHeader::new("Available Gigs")
                    .default_open(true)
                    .show(ui, |ui| {
                        if gig_catalog.is_empty() {
                            ui.label("No gigs available.");
                        }
                        for gig in &gig_catalog {
                            ui.group(|ui| {
                                ui.label(format!("{} ({:?})", gig.name, gig.category));
                                ui.label(format!(
                                    "Duration: {} weeks | Pay: ${}",
                                    gig.duration_weeks, gig.base_pay
                                ));
                                ui.label(format!(
                                    "Req: {:?} | Stress: +{}",
                                    gig.required_recognition_tier, gig.stress_cost
                                ));
                                if let Some(idx) = selected.0 {
                                    if ui.button("Assign").clicked() {
                                        pending_cmd = Some(GameCommand::AssignGig {
                                            artist_index: idx,
                                            gig: gig.clone(),
                                        });
                                    }
                                } else {
                                    ui.label("(Select an artist first)");
                                }
                            });
                        }
                    });

                ui.separator();

                // --- Recruitment ---
                egui::CollapsingHeader::new("Recruitment")
                    .default_open(false)
                    .show(ui, |ui| {
                        if prospects.is_empty() {
                            ui.label("No available prospects.");
                        }
                        for (i, prospect) in prospects.iter().enumerate() {
                            ui.group(|ui| {
                                ui.label(format!(
                                    "{} (Age {})",
                                    prospect.definition.name, prospect.definition.starting_age
                                ));
                                ui.label(format!(
                                    "Location: {} | Day: {}",
                                    prospect.location, prospect.available_day
                                ));
                                ui.label(format!(
                                    "Commission: {}%",
                                    (prospect.base_commission * 100.0) as i32
                                ));
                                if prospect.is_locked(total_weeks) {
                                    ui.label("Locked out");
                                } else if artist_count < max_artists {
                                    if ui.button("Sign Artist").clicked() {
                                        pending_cmd = Some(GameCommand::SignArtist {
                                            prospect_index: i,
                                            commission_adjustment: 0,
                                        });
                                    }
                                } else {
                                    ui.label("(Roster full)");
                                }
                            });
                        }
                    });

                ui.separator();

                // --- Office Upgrades ---
                egui::CollapsingHeader::new("Office")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(format!("Current: {:?}", office_tier));
                        if let Some(next) = office::next_upgrade(office_tier, &office_upgrades) {
                            ui.label(format!("Upgrade to {:?}: ${}", next.tier, next.cost.0));
                            ui.label(format!(
                                "  +{} artist slots, {}% training discount",
                                next.max_artists_bonus, next.training_cost_discount_pct
                            ));
                            if office::can_afford(balance, next) {
                                if ui.button("Upgrade Office").clicked() {
                                    pending_cmd = Some(GameCommand::UpgradeOffice);
                                }
                            } else {
                                ui.label("(Insufficient funds)");
                            }
                        } else {
                            ui.label("Max tier reached!");
                        }
                        if office_tier > OfficeTier::Starter {
                            let refund = office::downgrade_refund(office_tier, &office_upgrades);
                            if ui
                                .button(format!("Downgrade (refund ${})", refund.0))
                                .clicked()
                            {
                                pending_cmd = Some(GameCommand::DowngradeOffice);
                            }
                        }
                    });

                ui.separator();

                // --- Outfit Shop ---
                egui::CollapsingHeader::new("Outfits")
                    .default_open(false)
                    .show(ui, |ui| {
                        if outfit_catalog.is_empty() {
                            ui.label("No outfits in shop.");
                        }
                        for outfit in &outfit_catalog {
                            ui.group(|ui| {
                                ui.label(&outfit.name);
                                ui.label(format!("Cost: ${}", outfit.cost.0));
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
                                    ui.label("Owned");
                                    if let Some(idx) = selected.0
                                        && ui.button("Equip").clicked()
                                    {
                                        pending_cmd = Some(GameCommand::EquipOutfit {
                                            artist_index: idx,
                                            outfit_id: outfit.id,
                                        });
                                    }
                                } else if ui.button("Buy").clicked() {
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
