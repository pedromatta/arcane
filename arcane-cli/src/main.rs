mod cli;
mod commands;

use crate::cli::{CategoryCommands, Cli, Commands, ScheduleCommands};
use crate::commands::add::add_category;
use crate::commands::list::list_categories;
use crate::commands::remove::remove_category_cmd;
use crate::commands::schedule::{add_slot, list_slots, remove_slot};
use crate::commands::setup::{get_config, initialize_app};
use crate::commands::import::import_cmd;
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
                category,
                time,
                days,
            } => {
                add_slot(&pool, category, time, days).await;
            }
            ScheduleCommands::List => {
                list_slots(&pool).await;
            }
            ScheduleCommands::Remove { id } => {
                remove_slot(&pool, *id).await;
            }
        },
        Some(Commands::Import { path }) => {
            import_cmd(&pool, path).await;
        }
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

    #[tokio::test]
    async fn test_cli_import() {
        let pool = initialize_app("sqlite::memory:").await.unwrap();
        
        let manifest_path = std::env::current_dir()
            .unwrap()
            .join("test_manifest.toml");
        
        let manifest_content = r#"
            [[category]]
            name = "Rust CLI Test"
            default_minutes = 45
            color = "blue"

            [[schedule]]
            time = "14:00"
            category = "Rust CLI Test"
            days = 31
        "#;
        std::fs::write(&manifest_path, manifest_content).unwrap();

        import_cmd(&pool, manifest_path.to_str().unwrap()).await;

        let cats = arcane_core::db::category::list_categories(&pool).await.unwrap();
        assert_eq!(cats.len(), 1);
        assert_eq!(cats[0].name, "Rust CLI Test");
        assert_eq!(cats[0].default_minutes, 45);
        assert_eq!(cats[0].color, "blue");

        let slots = arcane_core::db::schedule::list_schedule_slots_detail(&pool).await.unwrap();
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].category_name, "Rust CLI Test");
        assert_eq!(slots[0].time_of_day, "14:00");
        assert_eq!(slots[0].days_of_week, 31);

        let _ = std::fs::remove_file(manifest_path);
    }
}
