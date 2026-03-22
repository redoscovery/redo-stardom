use serde::{Deserialize, Serialize};

use crate::persona::{ImageTag, ImageTags, IMAGE_TAG_MAX, IMAGE_TAG_MIN};
use crate::types::{Money, OutfitId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraitModifier {
    Confidence,
    Rebellion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutfitDef {
    pub id: OutfitId,
    pub name: String,
    pub cost: Money,
    pub image_modifiers: Vec<(ImageTag, i32)>,
    pub trait_modifiers: Vec<(TraitModifier, i32)>,
}

impl OutfitDef {
    pub fn apply_to_image(&self, base: &ImageTags) -> ImageTags {
        let mut result = *base;
        for (tag, modifier) in &self.image_modifiers {
            let current = tag.value_from(&result);
            let new_val = (current + modifier).clamp(IMAGE_TAG_MIN, IMAGE_TAG_MAX);
            set_image_tag(&mut result, *tag, new_val);
        }
        result
    }
}

fn set_image_tag(tags: &mut ImageTags, tag: ImageTag, value: i32) {
    match tag {
        ImageTag::Pure => tags.pure = value,
        ImageTag::Sexy => tags.sexy = value,
        ImageTag::Cool => tags.cool = value,
        ImageTag::Intellectual => tags.intellectual = value,
        ImageTag::Funny => tags.funny = value,
        ImageTag::Mysterious => tags.mysterious = value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_outfit(image_modifiers: Vec<(ImageTag, i32)>) -> OutfitDef {
        OutfitDef {
            id: OutfitId(1),
            name: "Test Outfit".to_string(),
            cost: Money(5000),
            image_modifiers,
            trait_modifiers: vec![],
        }
    }

    #[test]
    fn apply_outfit_modifiers() {
        let base = ImageTags {
            pure: 40,
            intellectual: 20,
            ..Default::default()
        };
        let outfit = make_outfit(vec![
            (ImageTag::Pure, 15),
            (ImageTag::Intellectual, 10),
        ]);
        let result = outfit.apply_to_image(&base);
        assert_eq!(result.pure, 55);
        assert_eq!(result.intellectual, 30);
    }

    #[test]
    fn outfit_modifiers_clamp_to_max() {
        let base = ImageTags {
            sexy: 80,
            ..Default::default()
        };
        let outfit = make_outfit(vec![(ImageTag::Sexy, 50)]);
        let result = outfit.apply_to_image(&base);
        assert_eq!(result.sexy, 100);
    }

    #[test]
    fn serialization_roundtrip() {
        let outfit = OutfitDef {
            id: OutfitId(1),
            name: "Stage Dress".to_string(),
            cost: Money(12000),
            image_modifiers: vec![(ImageTag::Pure, 10), (ImageTag::Sexy, -5)],
            trait_modifiers: vec![(TraitModifier::Confidence, 5)],
        };
        let serialized = ron::to_string(&outfit).unwrap();
        let deserialized: OutfitDef = ron::from_str(&serialized).unwrap();
        assert_eq!(deserialized.id, outfit.id);
        assert_eq!(deserialized.name, outfit.name);
        assert_eq!(deserialized.cost, outfit.cost);
        assert_eq!(deserialized.image_modifiers.len(), outfit.image_modifiers.len());
        assert_eq!(deserialized.trait_modifiers.len(), outfit.trait_modifiers.len());
    }
}
