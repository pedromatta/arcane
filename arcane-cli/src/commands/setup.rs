use anyhow::{Context, Result};
use arcane_core::config::{AppConfig, get_default_config_path};
use arcane_core::db::establish_connection;
use sqlx::SqlitePool;

pub async fn initialize_app(db_url: &str) -> Result<SqlitePool> {
    let pool = establish_connection(db_url)
        .await
        .context("Failed to establish database connection and apply migrations")?;
    Ok(pool)
}

pub fn get_config(cli_config: Option<std::path::PathBuf>) -> Result<AppConfig> {
    let config_path = cli_config
        .or_else(|| get_default_config_path())
        .context("Could not determine configuration path")?;

    let config_content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file at {:?}", config_path))?;

    toml::from_str(&config_content).context("Failed to parse config file as TOML")
}
