mod helpers;
mod models;
use clap::Parser;

use crate::helpers::add::add_category;
use crate::helpers::init::{get_config, initialize_app};
use crate::helpers::list::list_categories;

use crate::models::category_commands::CategoryCommands;
use crate::models::cli::Cli;
use crate::models::commands::Commands;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config = get_config(cli.config.clone());

    let db_url = config.general.database_path.unwrap();

    let pool = initialize_app(&db_url).await.unwrap();

    match &cli.command {
        Some(Commands::Init) => {
            initialize_app(&db_url)
                .await
                .expect("Failed to initialize database");
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
        _ => {}
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
