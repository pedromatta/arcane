mod cli;
mod commands;

use crate::cli::{CategoryCommands, Cli, Commands, ScheduleCommands};
use crate::commands::add::add_category;
use crate::commands::list::list_categories;
use crate::commands::remove::remove_category_cmd;
use crate::commands::schedule::{add_slot, list_slots, remove_slot};
use crate::commands::setup::{get_config, initialize_app};
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = get_config(cli.config)?;

    let db_url = config
        .general
        .database_path
        .ok_or_else(|| anyhow::anyhow!("database_path not set in configuration"))?;

    let pool = initialize_app(&db_url).await?;

    match &cli.command {
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
            CategoryCommands::Remove { name } => {
                remove_category_cmd(&pool, name).await;
            }
        },
        Some(Commands::Schedule { subcommand }) => match subcommand {
            ScheduleCommands::Add {
                category_id,
                time,
                days,
            } => {
                add_slot(&pool, *category_id, time, *days).await;
            }
            ScheduleCommands::List => {
                list_slots(&pool).await;
            }
            ScheduleCommands::Remove { id } => {
                remove_slot(&pool, *id).await;
            }
        },
        _ => {
            println!("Welcome to Arcane! Use --help to list commands.");
        }
    }

    Ok(())
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
