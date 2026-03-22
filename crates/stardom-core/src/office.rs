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

/// Assumes upgrades slice is sorted by tier ascending.
pub fn next_upgrade<'a>(current: OfficeTier, upgrades: &'a [OfficeUpgradeDef]) -> Option<&'a OfficeUpgradeDef> {
    upgrades.iter().find(|u| u.tier > current)
}

pub fn can_afford(balance: Money, upgrade: &OfficeUpgradeDef) -> bool {
    balance.0 >= upgrade.cost.0
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_upgrades() -> Vec<OfficeUpgradeDef> {
        vec![
            OfficeUpgradeDef {
                tier: OfficeTier::Standard,
                cost: Money(500_000),
                max_artists_bonus: 2,
                training_cost_discount_pct: 5,
                weekly_upkeep: Money(2_000),
            },
            OfficeUpgradeDef {
                tier: OfficeTier::Premium,
                cost: Money(2_000_000),
                max_artists_bonus: 5,
                training_cost_discount_pct: 10,
                weekly_upkeep: Money(8_000),
            },
            OfficeUpgradeDef {
                tier: OfficeTier::Luxury,
                cost: Money(5_000_000),
                max_artists_bonus: 10,
                training_cost_discount_pct: 20,
                weekly_upkeep: Money(20_000),
            },
        ]
    }

    #[test]
    fn next_upgrade_from_starter() {
        let upgrades = make_upgrades();
        let next = next_upgrade(OfficeTier::Starter, &upgrades).unwrap();
        assert_eq!(next.tier, OfficeTier::Standard);
    }

    #[test]
    fn next_upgrade_from_luxury_is_none() {
        let upgrades = make_upgrades();
        assert!(next_upgrade(OfficeTier::Luxury, &upgrades).is_none());
    }

    #[test]
    fn can_afford_upgrade() {
        let upgrades = make_upgrades();
        let standard = &upgrades[0]; // cost 500K
        assert!(can_afford(Money(600_000), standard));
        assert!(!can_afford(Money(400_000), standard));
    }

    #[test]
    fn downgrade_returns_partial_cost() {
        let upgrades = make_upgrades();
        // Standard costs 500K → 40% refund = 200K
        let refund = downgrade_refund(OfficeTier::Standard, &upgrades);
        assert_eq!(refund, Money(200_000));
    }

    #[test]
    fn downgrade_from_starter_returns_zero() {
        let upgrades = make_upgrades();
        // Starter is not in upgrades list → Money(0)
        let refund = downgrade_refund(OfficeTier::Starter, &upgrades);
        assert_eq!(refund, Money(0));
    }

    #[test]
    fn weekly_upkeep_for_tier() {
        let upgrades = make_upgrades();
        // Starter not in list → 0
        assert_eq!(get_weekly_upkeep(OfficeTier::Starter, &upgrades), Money(0));
        // Standard → 2000
        assert_eq!(get_weekly_upkeep(OfficeTier::Standard, &upgrades), Money(2_000));
    }

    #[test]
    fn serialization_roundtrip() {
        let def = OfficeUpgradeDef {
            tier: OfficeTier::Premium,
            cost: Money(2_000_000),
            max_artists_bonus: 5,
            training_cost_discount_pct: 10,
            weekly_upkeep: Money(8_000),
        };
        let serialized = ron::to_string(&def).unwrap();
        let deserialized: OfficeUpgradeDef = ron::from_str(&serialized).unwrap();
        assert_eq!(deserialized.tier, def.tier);
        assert_eq!(deserialized.cost, def.cost);
        assert_eq!(deserialized.max_artists_bonus, def.max_artists_bonus);
        assert_eq!(deserialized.training_cost_discount_pct, def.training_cost_discount_pct);
        assert_eq!(deserialized.weekly_upkeep, def.weekly_upkeep);
    }
}
