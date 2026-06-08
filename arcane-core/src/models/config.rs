use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub general: GeneralConfig,
    #[serde(default)]
    pub category: Vec<CategoryConfig>,
    pub schedule: Option<Vec<ScheduleConfig>>,
}

#[derive(Deserialize, Debug)]
pub struct GeneralConfig {
    pub database_path: Option<String>,
    #[serde(default = "default_notifications")]
    pub notifications_enabled: bool,
}

fn default_notifications() -> bool {
    true
}

#[derive(Deserialize, Debug)]
pub struct CategoryConfig {
    pub name: String,
    pub default_minutes: u32,
    pub color: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ScheduleConfig {
    pub time: String,
    pub category: String,
    pub days: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_toml_schema() {
        let raw_toml = r#"
            [general]
            database_path = "~/.local/share/arcane/arcane.db"     
            notifications_enabled = false

            [[category]]
            name = "Rust"
            default_minutes = 60
            color = "magenta"
            
            [[schedule]]
            time = "10:00"
            category = "Rust"
            days = 127
        "#;

        let config: AppConfig = toml::from_str(raw_toml).expect("Failed to parse valid TOML");

        assert_eq!(
            config.general.database_path.unwrap(),
            "~/.local/share/arcane/arcane.db"
        );
        assert!(!config.general.notifications_enabled);

        assert_eq!(config.category.len(), 1);
        assert_eq!(config.category[0].name, "Rust");
        assert_eq!(config.category[0].default_minutes, 60);

        assert_eq!(config.schedule.clone().unwrap().len(), 1);
        assert_eq!(config.schedule.clone().as_ref().unwrap()[0].days, 127);
    }

    #[test]
    fn applies_default_values_for_missing_fields() {
        let raw_toml = r#"
            [general]    

            [[category]]
            name = "Reading"
            default_minutes = 30
            color = "blue"
        "#;

        let config: AppConfig = toml::from_str(raw_toml).expect("Failed to parse valid TOML");

        assert_eq!(config.general.database_path, None);

        assert!(config.general.notifications_enabled);

        assert_eq!(config.category.len(), 1);
        assert_eq!(config.category[0].name, "Reading");
        assert_eq!(config.category[0].default_minutes, 30);
        assert_eq!(config.category[0].color, "blue");
    }
}
