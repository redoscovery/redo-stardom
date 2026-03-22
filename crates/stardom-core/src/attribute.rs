use serde::{Deserialize, Serialize};

macro_rules! clamp_fields {
    ($self:expr, $min:expr, $max:expr, $($field:ident),+) => {
        $( $self.$field = $self.$field.clamp($min, $max); )+
    };
}
pub(crate) use clamp_fields;

pub const BASE_ATTR_MIN: i32 = 1;
pub const BASE_ATTR_MAX: i32 = 100;
pub const SKILL_MIN: i32 = 0;
pub const SKILL_MAX: i32 = 10_000;
pub const TRAIT_MIN: i32 = 0;
pub const TRAIT_MAX: i32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseAttributes {
    pub stamina: i32,   // 體能
    pub intellect: i32, // 智識
    pub empathy: i32,   // 共情
    pub charm: i32,     // 魅力
}

impl Default for BaseAttributes {
    fn default() -> Self {
        Self {
            stamina: 50,
            intellect: 50,
            empathy: 50,
            charm: 50,
        }
    }
}

impl BaseAttributes {
    pub fn new(stamina: i32, intellect: i32, empathy: i32, charm: i32) -> Self {
        let mut a = Self {
            stamina,
            intellect,
            empathy,
            charm,
        };
        a.clamp();
        a
    }

    pub fn clamp(&mut self) {
        clamp_fields!(
            self,
            BASE_ATTR_MIN,
            BASE_ATTR_MAX,
            stamina,
            intellect,
            empathy,
            charm
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ProfessionalSkills {
    pub vocal: i32,      // 歌藝
    pub acting: i32,     // 演技
    pub dance: i32,      // 舞藝
    pub poise: i32,      // 儀態
    pub eloquence: i32,  // 口才
    pub creativity: i32, // 創作
}

impl ProfessionalSkills {
    pub fn clamp(&mut self) {
        clamp_fields!(
            self, SKILL_MIN, SKILL_MAX, vocal, acting, dance, poise, eloquence, creativity
        );
    }

    pub fn get(&self, target: crate::training::SkillTarget) -> i32 {
        use crate::training::SkillTarget;
        match target {
            SkillTarget::Vocal => self.vocal,
            SkillTarget::Acting => self.acting,
            SkillTarget::Dance => self.dance,
            SkillTarget::Poise => self.poise,
            SkillTarget::Eloquence => self.eloquence,
            SkillTarget::Creativity => self.creativity,
        }
    }

    pub fn get_mut(&mut self, target: crate::training::SkillTarget) -> &mut i32 {
        use crate::training::SkillTarget;
        match target {
            SkillTarget::Vocal => &mut self.vocal,
            SkillTarget::Acting => &mut self.acting,
            SkillTarget::Dance => &mut self.dance,
            SkillTarget::Poise => &mut self.poise,
            SkillTarget::Eloquence => &mut self.eloquence,
            SkillTarget::Creativity => &mut self.creativity,
        }
    }

    pub fn apply_gain(&mut self, target: crate::training::SkillTarget, amount: i32) {
        let field = self.get_mut(target);
        *field = (*field + amount).min(SKILL_MAX);
    }

    pub fn apply_loss(&mut self, target: crate::training::SkillTarget, amount: i32) {
        let field = self.get_mut(target);
        *field = (*field - amount).max(SKILL_MIN);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InnerTraits {
    pub confidence: i32, // 自信
    pub rebellion: i32,  // 叛逆
}

impl Default for InnerTraits {
    fn default() -> Self {
        Self {
            confidence: 50,
            rebellion: 30,
        }
    }
}

impl InnerTraits {
    pub fn clamp(&mut self) {
        clamp_fields!(self, TRAIT_MIN, TRAIT_MAX, confidence, rebellion);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_attributes_clamp_to_range() {
        let mut attrs = BaseAttributes::default();
        attrs.stamina = 150;
        attrs.clamp();
        assert_eq!(attrs.stamina, 100);
    }

    #[test]
    fn base_attributes_default_values() {
        let attrs = BaseAttributes::default();
        assert_eq!(attrs.stamina, 50);
        assert_eq!(attrs.intellect, 50);
        assert_eq!(attrs.empathy, 50);
        assert_eq!(attrs.charm, 50);
    }

    #[test]
    fn professional_skills_start_at_zero() {
        let skills = ProfessionalSkills::default();
        assert_eq!(skills.vocal, 0);
        assert_eq!(skills.acting, 0);
    }

    #[test]
    fn professional_skills_clamp_to_max() {
        let mut skills = ProfessionalSkills::default();
        skills.vocal = 12_000;
        skills.clamp();
        assert_eq!(skills.vocal, 10_000);
    }

    #[test]
    fn inner_traits_default() {
        let traits = InnerTraits::default();
        assert_eq!(traits.confidence, 50);
        assert_eq!(traits.rebellion, 30);
    }

    #[test]
    fn inner_traits_clamp() {
        let mut traits = InnerTraits {
            confidence: 120,
            rebellion: -5,
        };
        traits.clamp();
        assert_eq!(traits.confidence, 100);
        assert_eq!(traits.rebellion, 0);
    }

    #[test]
    fn base_attributes_lower_bound_clamp() {
        // stamina=0 is below BASE_ATTR_MIN (1), so BaseAttributes::new should clamp it to 1
        let attrs = BaseAttributes::new(0, 50, 50, 50);
        assert_eq!(attrs.stamina, BASE_ATTR_MIN);
        assert_eq!(attrs.intellect, 50);
        assert_eq!(attrs.empathy, 50);
        assert_eq!(attrs.charm, 50);
    }

    #[test]
    fn serialization_roundtrip() {
        let attrs = BaseAttributes::new(70, 60, 80, 55);
        let serialized = ron::to_string(&attrs).unwrap();
        let deserialized: BaseAttributes = ron::from_str(&serialized).unwrap();
        assert_eq!(attrs, deserialized);
    }
}
