mod add;
mod list;
use anyhow::Result;
use arcane_core::db::establish_connection;
use arcane_core::error::ArcaneError;
use clap::{Parser, Subcommand};
use sqlx::SqlitePool;
use std::env;

use crate::add::add_category;
use crate::list::list_categories;

pub async fn initialize_app(db_url: &str) -> Result<SqlitePool, ArcaneError> {
    let pool = establish_connection(db_url).await?;

    Ok(pool)
}

#[derive(Parser)]
#[clap(
    name = "Arcane",
    version = "0.1",
    author = "pedromatta",
    about = "A Rust routine planner"
)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the database and run embedded migrations
    Init,
    Log,
    Categories {
        #[command(subcommand)]
        subcommand: CategoryCommands,
    },
}

#[derive(Subcommand)]
enum CategoryCommands {
    /// Add a new category
    Add {
        /// Name of the category
        #[arg(short, long)]
        name: String,

        /// Default duration for tasks in this category (in minutes)
        #[arg(short, long, default_value_t = 25)]
        default_minutes: u32,

        /// Color of the category (hex code, eg. #FF5733)
        #[arg(short, long, default_value = "#FFFFFF")]
        color: String,
    },
    /// List all categories
    List,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    let pool = initialize_app(&db_url).await.unwrap();

    match &cli.command {
        Some(Commands::Init) => {
            dotenvy::dotenv().ok();
            initialize_app(&db_url)
                .await
                .expect("Failed to initialize database");
            println!("Database initialized successfully.");
        }
        Some(Commands::Log) => {
            println!("Log command selected. (Not implemented yet)");
        }
        Some(Commands::Categories { subcommand }) => match subcommand {
            CategoryCommands::Add {
                name,
                default_minutes,
                color,
            } => {
                add_category(&pool, name, default_minutes, color).await;
            }
            CategoryCommands::List => {
                list_categories(&pool).await;
            }
        },
        None => {}
    }
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
