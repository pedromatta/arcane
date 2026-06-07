use crate::error::ArcaneError;
use std::str::FromStr;

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

pub async fn establish_connection(database_url: &str) -> Result<SqlitePool, ArcaneError> {
    let connection_options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connection_options)
        .await?;

    sqlx::migrate!("./src/db/migrations/").run(&pool).await?;

    println!("Database connection established and migrations applied successfully.");
    Ok(pool)
}
