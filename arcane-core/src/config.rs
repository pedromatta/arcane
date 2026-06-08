use serde::Deserialize;
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use crate::error::ArcaneError;

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub general: GeneralConfig,
    #[serde(default, alias = "category", alias = "categories")]
    pub categories: Vec<CategoryConfig>,
    pub schedule: Option<Vec<ScheduleConfig>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GeneralConfig {
    pub database_path: Option<String>,
    #[serde(default = "default_notifications")]
    pub notifications_enabled: bool,
}

fn default_notifications() -> bool {
    true
}

#[derive(Deserialize, Debug, Clone)]
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

pub fn get_args_config_path(args: &str) -> Option<PathBuf> {
    let path = PathBuf::from(args);
    if path.is_absolute() {
        Some(path)
    } else {
        std::env::current_dir().ok().map(|cwd| cwd.join(path))
    }
}

pub fn get_env_config_path() -> Option<PathBuf> {
    if let Ok(env_path) = std::env::var("ARCANE_CONFIG_FILE") {
        let path = PathBuf::from(env_path);
        if path.is_absolute() {
            Some(path)
        } else {
            std::env::current_dir().ok().map(|cwd| cwd.join(path))
        }
    } else {
        None
    }
}

pub fn get_default_config_path() -> Option<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "Arcane", "arcane")?;
    let config_dir = proj_dirs.config_dir();

    generate_default_config(config_dir).ok()?;

    Some(config_dir.join("config.toml"))
}

fn generate_default_config(config_dir: &Path) -> Result<(), ArcaneError> {
    if !config_dir.exists() {
        std::fs::create_dir_all(config_dir)?;
    }
    let config_path = config_dir.join("config.toml");
    if !config_path.exists() {
        let default_db_path = get_default_db_path()
            .ok_or_else(|| ArcaneError::ConfigValidation("Failed to determine default database path".to_string()))?
            .to_string_lossy()
            .into_owned();
        let default_config = format!(
            "[general]\ndatabase_path = \"{}\"\nnotifications_enabled = true\n",
            default_db_path
        );
        std::fs::write(config_path, default_config)?;
    }
    Ok(())
}

fn get_default_db_path() -> Option<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "Arcane", "arcane")?;
    let data_dir = proj_dirs.data_dir();
    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir).ok()?;
    }
    Some(data_dir.join("arcane.db"))
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

        assert_eq!(config.categories.len(), 1);
        assert_eq!(config.categories[0].name, "Rust");
        assert_eq!(config.categories[0].default_minutes, 60);

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

        assert_eq!(config.categories.len(), 1);
        assert_eq!(config.categories[0].name, "Reading");
        assert_eq!(config.categories[0].default_minutes, 30);
        assert_eq!(config.categories[0].color, "blue");
    }
}
