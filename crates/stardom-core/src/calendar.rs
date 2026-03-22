use serde::{Deserialize, Serialize};

pub const WEEKS_PER_YEAR: u32 = 52;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Calendar {
    pub year: u32,
    pub week: u32, // 1-based, 1..=52
    pub total_weeks_elapsed: u32,
    pub goal_years: u32,
}

impl Calendar {
    pub fn new(goal_years: u32) -> Self {
        Self {
            year: 1,
            week: 1,
            total_weeks_elapsed: 0,
            goal_years,
        }
    }

    pub fn advance_week(&mut self) {
        self.total_weeks_elapsed += 1;
        self.week += 1;
        if self.week > WEEKS_PER_YEAR {
            self.week = 1;
            self.year += 1;
        }
    }

    pub fn is_goal_period_over(&self) -> bool {
        self.total_weeks_elapsed >= self.goal_years * WEEKS_PER_YEAR
    }

    pub fn is_rotation_a(&self) -> bool {
        self.week % 2 == 1
    }

    pub fn approximate_month(&self) -> u32 {
        ((self.week - 1) * 12 / WEEKS_PER_YEAR) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_calendar_starts_at_week_1_year_1() {
        let cal = Calendar::new(3);
        assert_eq!(cal.year, 1);
        assert_eq!(cal.week, 1);
        assert_eq!(cal.total_weeks_elapsed, 0);
        assert_eq!(cal.goal_years, 3);
    }

    #[test]
    fn advance_week() {
        let mut cal = Calendar::new(3);
        cal.advance_week();
        assert_eq!(cal.week, 2);
        assert_eq!(cal.total_weeks_elapsed, 1);
        assert_eq!(cal.year, 1);
    }

    #[test]
    fn year_rolls_over_at_week_52() {
        let mut cal = Calendar::new(3);
        for _ in 0..52 {
            cal.advance_week();
        }
        assert_eq!(cal.year, 2);
        assert_eq!(cal.week, 1);
        assert_eq!(cal.total_weeks_elapsed, 52);
    }

    #[test]
    fn goal_reached_after_3_years() {
        let mut cal = Calendar::new(3);
        assert!(!cal.is_goal_period_over());
        for _ in 0..156 {
            cal.advance_week();
        }
        assert!(cal.is_goal_period_over());
    }

    #[test]
    fn is_biweekly_rotation() {
        let mut cal = Calendar::new(3);
        // week 1 → rotation A
        assert!(cal.is_rotation_a());
        cal.advance_week();
        // week 2 → not rotation A
        assert!(!cal.is_rotation_a());
    }

    #[test]
    fn approximate_month() {
        let cal = Calendar::new(3);
        // week 1 → month 1 (January)
        assert_eq!(cal.approximate_month(), 1);

        let mut cal36 = Calendar::new(3);
        // advance to week 36
        for _ in 0..35 {
            cal36.advance_week();
        }
        assert_eq!(cal36.week, 36);
        // week 36 → September (9)
        assert_eq!(cal36.approximate_month(), 9);
    }
}
