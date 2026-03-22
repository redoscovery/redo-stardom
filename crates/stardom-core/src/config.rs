use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub goal_years: u32,
    pub retirement_age: u32,
    pub max_artists: u32,
    pub starting_balance: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            goal_years: 3,
            retirement_age: 40,
            max_artists: 3,
            starting_balance: 1_000_000,
        }
    }
}

impl Settings {
    pub fn load_from_str(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings() {
        let s = Settings::default();
        assert_eq!(s.goal_years, 3);
        assert_eq!(s.retirement_age, 40);
        assert_eq!(s.max_artists, 3);
        assert_eq!(s.starting_balance, 1_000_000);
    }

    #[test]
    fn load_from_toml_string() {
        let toml_str = r#"
goal_years = 5
retirement_age = 45
max_artists = 6
starting_balance = 2000000
"#;
        let s = Settings::load_from_str(toml_str).unwrap();
        assert_eq!(s.goal_years, 5);
        assert_eq!(s.retirement_age, 45);
        assert_eq!(s.max_artists, 6);
        assert_eq!(s.starting_balance, 2_000_000);
    }
}
