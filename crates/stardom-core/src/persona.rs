use crate::attribute::clamp_fields;
use serde::{Deserialize, Serialize};

pub const SPECTRUM_MIN: i32 = -100;
pub const SPECTRUM_MAX: i32 = 100;
pub const IMAGE_TAG_MIN: i32 = 0;
pub const IMAGE_TAG_MAX: i32 = 100;
const MAX_SPECTRUM_BONUS: f64 = 0.15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spectrum {
    Social,
    Thinking,
    Action,
    Stance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PersonalitySpectrums {
    pub social: i32,   // -100 Introvert to +100 Extrovert
    pub thinking: i32, // -100 Intuitive to +100 Logical
    pub action: i32,   // -100 Cautious to +100 Adventurous
    pub stance: i32,   // -100 Easygoing to +100 Competitive
}

impl PersonalitySpectrums {
    pub fn clamp(&mut self) {
        clamp_fields!(
            self,
            SPECTRUM_MIN,
            SPECTRUM_MAX,
            social,
            thinking,
            action,
            stance
        );
    }

    /// Returns absolute modifier strength (0.0 to 0.15) for given spectrum.
    pub fn modifier(&self, spectrum: Spectrum) -> f64 {
        let value = self.get(spectrum);
        (value.abs() as f64 / 100.0) * MAX_SPECTRUM_BONUS
    }

    pub fn get(&self, spectrum: Spectrum) -> i32 {
        match spectrum {
            Spectrum::Social => self.social,
            Spectrum::Thinking => self.thinking,
            Spectrum::Action => self.action,
            Spectrum::Stance => self.stance,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ImageTags {
    pub pure: i32,         // 清純
    pub sexy: i32,         // 性感
    pub cool: i32,         // 酷帥
    pub intellectual: i32, // 知性
    pub funny: i32,        // 搞笑
    pub mysterious: i32,   // 神秘
}

impl ImageTags {
    pub fn clamp(&mut self) {
        clamp_fields!(
            self,
            IMAGE_TAG_MIN,
            IMAGE_TAG_MAX,
            pure,
            sexy,
            cool,
            intellectual,
            funny,
            mysterious
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spectrum_clamp_to_range() {
        let mut spectrums = PersonalitySpectrums {
            social: 150,
            thinking: -200,
            action: 0,
            stance: 0,
        };
        spectrums.clamp();
        assert_eq!(spectrums.social, 100);
        assert_eq!(spectrums.thinking, -100);
    }

    #[test]
    fn spectrum_modifier_at_extremes() {
        let spectrums = PersonalitySpectrums {
            social: 100,
            thinking: 0,
            action: -100,
            stance: 50,
        };
        let epsilon = 1e-9;
        assert!((spectrums.modifier(Spectrum::Social) - 0.15).abs() < epsilon);
        assert!((spectrums.modifier(Spectrum::Thinking) - 0.0).abs() < epsilon);
        assert!((spectrums.modifier(Spectrum::Action) - 0.15).abs() < epsilon);
        assert!((spectrums.modifier(Spectrum::Stance) - 0.075).abs() < epsilon);
    }

    #[test]
    fn image_tags_independent() {
        let tags = ImageTags {
            pure: 80,
            sexy: 70,
            ..Default::default()
        };
        assert_eq!(tags.pure, 80);
        assert_eq!(tags.sexy, 70);
    }

    #[test]
    fn image_tags_clamp() {
        let mut tags = ImageTags {
            cool: 120,
            ..Default::default()
        };
        tags.clamp();
        assert_eq!(tags.cool, 100);
    }

    #[test]
    fn serialization_roundtrip() {
        let spectrums = PersonalitySpectrums {
            social: 75,
            thinking: -30,
            action: 50,
            stance: -80,
        };
        let serialized = ron::to_string(&spectrums).unwrap();
        let deserialized: PersonalitySpectrums = ron::from_str(&serialized).unwrap();
        assert_eq!(spectrums, deserialized);
    }
}
