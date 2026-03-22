use crate::types::Money;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficeTier {
    Starter,
    Standard,
    Premium,
    Luxury,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyState {
    pub balance: Money,
    pub office_tier: OfficeTier,
    pub max_artists: u32,
    pub consecutive_negative_weeks: u32,
}

impl CompanyState {
    pub fn new(starting_balance: Money, max_artists: u32) -> Self {
        Self {
            balance: starting_balance,
            office_tier: OfficeTier::Starter,
            max_artists,
            consecutive_negative_weeks: 0,
        }
    }

    /// Deduct money. Debt is allowed per spec A.13 — balance can go negative.
    pub fn spend(&mut self, amount: Money) {
        self.balance = self.balance - amount;
    }

    pub fn earn(&mut self, amount: Money) {
        self.balance = self.balance + amount;
    }

    /// Update bankruptcy counter at end of week.
    /// `has_pending_income`: true if any artist has a gig completing within 2 weeks.
    pub fn update_bankruptcy_counter(&mut self, has_pending_income: bool) {
        if self.balance.0 >= 0 {
            self.consecutive_negative_weeks = 0;
        } else if !has_pending_income {
            self.consecutive_negative_weeks += 1;
        }
    }

    pub fn is_bankrupt(&self) -> bool {
        self.balance.0 < 0 && self.consecutive_negative_weeks >= 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Money;

    #[test]
    fn new_company() {
        let company = CompanyState::new(Money(100_000), 3);
        assert_eq!(company.balance, Money(100_000));
        assert_eq!(company.office_tier, OfficeTier::Starter);
        assert_eq!(company.max_artists, 3);
        assert_eq!(company.consecutive_negative_weeks, 0);
    }

    #[test]
    fn spend_money() {
        let mut company = CompanyState::new(Money(100_000), 3);
        company.spend(Money(50_000));
        assert_eq!(company.balance, Money(50_000));
    }

    #[test]
    fn spend_can_go_negative() {
        let mut company = CompanyState::new(Money(10_000), 3);
        company.spend(Money(20_000));
        assert_eq!(company.balance, Money(-10_000));
    }

    #[test]
    fn bankruptcy_check_resets_on_positive() {
        let mut company = CompanyState::new(Money(-5_000), 3);
        company.consecutive_negative_weeks = 2;
        company.earn(Money(10_000)); // balance now positive
        company.update_bankruptcy_counter(false);
        assert_eq!(company.consecutive_negative_weeks, 0);
    }

    #[test]
    fn bankruptcy_triggers_at_4_weeks() {
        let mut company = CompanyState::new(Money(-1_000), 3);
        company.consecutive_negative_weeks = 3;
        company.update_bankruptcy_counter(false);
        assert_eq!(company.consecutive_negative_weeks, 4);
        assert!(company.is_bankrupt());
    }

    #[test]
    fn bankruptcy_paused_by_pending_income() {
        let mut company = CompanyState::new(Money(-1_000), 3);
        company.consecutive_negative_weeks = 3;
        company.update_bankruptcy_counter(true);
        assert_eq!(company.consecutive_negative_weeks, 3);
        assert!(!company.is_bankrupt());
    }

    #[test]
    fn office_tier_ordering() {
        assert!(OfficeTier::Starter < OfficeTier::Standard);
        assert!(OfficeTier::Standard < OfficeTier::Premium);
        assert!(OfficeTier::Premium < OfficeTier::Luxury);
    }
}
