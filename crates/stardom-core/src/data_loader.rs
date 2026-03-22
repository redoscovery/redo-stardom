use serde::{Deserialize, Serialize};
use crate::artist::Artist;
use crate::attribute::{BaseAttributes, InnerTraits};
use crate::persona::{ImageTags, PersonalitySpectrums};
use crate::stats::AuxiliaryStats;
use crate::types::{Activity, ArtistId};

/// Data definition for an artist, loaded from RON files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistDefinition {
    pub id: ArtistId,
    pub name: String,
    pub starting_age: u32,
    pub base_attributes: BaseAttributes,
    pub personality: PersonalitySpectrums,
    pub traits: InnerTraits,
    pub image: ImageTags,
}

impl ArtistDefinition {
    pub fn into_artist(self) -> Artist {
        Artist {
            id: self.id,
            name: self.name,
            age: self.starting_age,
            base_attributes: self.base_attributes,
            skills: Default::default(),
            traits: self.traits,
            personality: self.personality,
            image: self.image,
            stats: AuxiliaryStats::default(),
            current_activity: Activity::Idle,
            inactive_weeks: 0,
        }
    }
}

pub fn load_artist_definition(ron_str: &str) -> Result<ArtistDefinition, ron::error::SpannedError> {
    ron::from_str(ron_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RON: &str = r#"
        ArtistDefinition(
            id: ArtistId(1),
            name: "Luna Star",
            starting_age: 18,
            base_attributes: BaseAttributes(
                stamina: 60,
                intellect: 55,
                empathy: 70,
                charm: 80,
            ),
            personality: PersonalitySpectrums(
                social: 30,
                thinking: -20,
                action: 10,
                stance: -40,
            ),
            traits: InnerTraits(
                confidence: 55,
                rebellion: 25,
            ),
            image: ImageTags(
                pure: 60,
                sexy: 20,
                cool: 40,
                intellectual: 30,
                funny: 10,
                mysterious: 50,
            ),
        )
    "#;

    #[test]
    fn load_artist_definition_from_ron() {
        let def = load_artist_definition(SAMPLE_RON).unwrap();
        assert_eq!(def.name, "Luna Star");
        assert_eq!(def.starting_age, 18);
        assert_eq!(def.base_attributes.charm, 80);
        assert_eq!(def.personality.social, 30);
    }

    #[test]
    fn artist_definition_to_artist() {
        let def = load_artist_definition(SAMPLE_RON).unwrap();
        let artist = def.into_artist();
        assert_eq!(artist.id, ArtistId(1));
        assert_eq!(artist.age, 18);
        assert_eq!(artist.stats, AuxiliaryStats::default());
        assert_eq!(artist.current_activity, Activity::Idle);
        assert_eq!(artist.inactive_weeks, 0);
    }
}
