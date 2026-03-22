use crate::attribute::{BaseAttributes, InnerTraits, ProfessionalSkills};
use crate::persona::{ImageTags, PersonalitySpectrums};
use crate::stats::AuxiliaryStats;
use crate::types::{Activity, ArtistId, OutfitId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub id: ArtistId,
    pub name: String,
    pub age: u32,
    pub base_attributes: BaseAttributes,
    pub skills: ProfessionalSkills,
    pub traits: InnerTraits,
    pub personality: PersonalitySpectrums,
    pub image: ImageTags,
    pub stats: AuxiliaryStats,
    pub current_activity: Activity,
    pub inactive_weeks: u32,
    pub locked_weeks: u32,
    pub equipped_outfit: Option<OutfitId>,
    pub commission_rate: f32,
}

impl Artist {
    pub fn new(id: ArtistId, name: String, age: u32, base_attributes: BaseAttributes) -> Self {
        Self {
            id,
            name,
            age,
            base_attributes,
            skills: ProfessionalSkills::default(),
            traits: InnerTraits::default(),
            personality: PersonalitySpectrums::default(),
            image: ImageTags::default(),
            stats: AuxiliaryStats::default(),
            current_activity: Activity::Idle,
            inactive_weeks: 0,
            locked_weeks: 0,
            equipped_outfit: None,
            commission_rate: 0.30,
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locked_weeks > 0
    }

    pub fn is_retired(&self, retirement_age: u32) -> bool {
        self.age >= retirement_age
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::BaseAttributes;
    use crate::types::ArtistId;

    fn make_artist() -> Artist {
        let attrs = BaseAttributes::new(70, 60, 80, 55);
        Artist::new(ArtistId(1), "Yuki".to_string(), 22, attrs)
    }

    #[test]
    fn artist_creation() {
        let artist = make_artist();
        assert_eq!(artist.id, ArtistId(1));
        assert_eq!(artist.name, "Yuki");
        assert_eq!(artist.age, 22);
        assert_eq!(artist.base_attributes.stamina, 70);
        assert_eq!(artist.skills.vocal, 0);
        assert_eq!(artist.skills.acting, 0);
        assert_eq!(artist.current_activity, Activity::Idle);
        assert_eq!(artist.inactive_weeks, 0);
    }

    #[test]
    fn artist_serialization_roundtrip() {
        let artist = make_artist();
        let serialized = ron::to_string(&artist).unwrap();
        let deserialized: Artist = ron::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, artist.id);
        assert_eq!(deserialized.name, artist.name);
    }

    #[test]
    fn artist_locked_in_gig() {
        let mut artist = make_artist();
        assert!(!artist.is_locked());
        artist.locked_weeks = 3;
        assert!(artist.is_locked());
        artist.locked_weeks = 0;
        assert!(!artist.is_locked());
    }

    #[test]
    fn artist_is_retired_at_age_limit() {
        let attrs = BaseAttributes::default();
        let artist_39 = Artist::new(ArtistId(2), "Hana".to_string(), 39, attrs);
        let artist_40 = Artist::new(ArtistId(3), "Hana".to_string(), 40, attrs);
        assert!(!artist_39.is_retired(40));
        assert!(artist_40.is_retired(40));
    }
}
