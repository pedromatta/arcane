use arcane_core::config::{get_args_config_path, get_default_config_path, get_env_config_path};
use arcane_core::db::establish_connection;
use arcane_core::error::ArcaneError;
use arcane_core::models::config::AppConfig;
use sqlx::SqlitePool;

pub async fn initialize_app(db_url: &str) -> Result<SqlitePool, ArcaneError> {
    let pool = establish_connection(db_url).await?;
    Ok(pool)
}

pub fn get_config(cli_config: Option<String>) -> AppConfig {
    let config_path = if let Some(cli_path) = cli_config {
        get_args_config_path(cli_path)
    } else if std::env::var("ARCANE_CONFIG_FILE").is_ok() {
        get_env_config_path()
    } else {
        get_default_config_path()
    }
    .expect("Could not determine configuration path");

    let config_content = std::fs::read_to_string(&config_path)
        .expect("Failed to read config.toml. Please ensure it exists and is readable.");

    toml::from_str(&config_content).expect("Failed to parse valid TOML")
}
