use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

macro_rules! id_newtype {
    ($($name:ident),+) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
            pub struct $name(pub u32);
        )+
    };
}

id_newtype!(ArtistId, GigId, OutfitId, CrisisId);

/// Game currency. Stored as i64 to allow negative balances (debt).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Money(pub i64);

impl Add for Money {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Money(self.0 + rhs.0)
    }
}

impl Sub for Money {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Money(self.0 - rhs.0)
    }
}

/// What an artist is doing this week.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activity {
    Training,
    PartTimeJob,
    Gig,
    Rest,
    Idle,
}

impl Activity {
    /// Whether this activity counts as public exposure (prevents popularity inactivity penalty).
    pub fn is_public(&self) -> bool {
        matches!(self, Activity::Gig | Activity::PartTimeJob)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artist_id_equality() {
        assert_eq!(ArtistId(1), ArtistId(1));
        assert_ne!(ArtistId(1), ArtistId(2));
    }

    #[test]
    fn artist_id_serialization() {
        let id = ArtistId(42);
        let serialized = ron::to_string(&id).unwrap();
        let deserialized: ArtistId = ron::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn money_arithmetic() {
        let a = Money(1000);
        let b = Money(500);
        assert_eq!(a + b, Money(1500));
        assert_eq!(a - b, Money(500));
    }

    #[test]
    fn money_can_be_negative() {
        let a = Money(1000);
        let b = Money(5000);
        assert_eq!(a - b, Money(-4000));
    }

    #[test]
    fn activity_is_public() {
        assert!(Activity::Gig.is_public());
        assert!(Activity::PartTimeJob.is_public());
        assert!(!Activity::Training.is_public());
        assert!(!Activity::Rest.is_public());
        assert!(!Activity::Idle.is_public());
    }
}
