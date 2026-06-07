use anyhow::{Context, Result};
use arcane_core::db::establish_connection;
use std::env;

pub async fn initialize_app(db_url: &str) -> Result<()> {
    let _pool = establish_connection(db_url)
        .await
        .context("Failed to initialize database pool and run embedded migrations");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().expect("Failed to load .env file");

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    initialize_app(&db_url).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_initialization() {
        let result = initialize_app("sqlite::memory:").await;
        assert!(result.is_ok());
    }
}
