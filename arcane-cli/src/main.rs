mod cli;
mod commands;

use crate::cli::{CategoryCommands, Cli, Commands, ScheduleCommands};
use crate::commands::add::add_category;
use crate::commands::list::list_categories;
use crate::commands::remove::remove_category_cmd;
use crate::commands::schedule::{add_slot, list_slots, remove_slot};
use crate::commands::setup::{get_config, initialize_app};
use crate::commands::import::import_cmd;
use crate::commands::export::export_cmd;
use crate::commands::tonight::tonight_cmd;
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
        Some(Commands::Export) => {
            export_cmd(&pool).await;
        }
        Some(Commands::Tonight { time, category }) => {
            tonight_cmd(&pool, time, category).await;
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

    #[tokio::test]
    async fn test_cli_export() {
        let pool = initialize_app("sqlite::memory:").await.unwrap();
        
        let manifest_path = std::env::current_dir()
            .unwrap()
            .join("test_export_manifest.toml");
        
        let manifest_content = r#"[[category]]
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

        let exported = arcane_core::db::schedule::export_manifest(&pool).await.unwrap();
        let toml_str = toml::to_string(&exported).unwrap();
        
        // Assert that the serialized output matches key parts of the TOML structure
        assert!(toml_str.contains("name = \"Rust CLI Test\""));
        assert!(toml_str.contains("default_minutes = 45"));
        assert!(toml_str.contains("color = \"blue\""));
        assert!(toml_str.contains("time = \"14:00\""));
        assert!(toml_str.contains("category = \"Rust CLI Test\""));
        assert!(toml_str.contains("days = 31"));

        // Call export_cmd to verify execution
        export_cmd(&pool).await;

        let _ = std::fs::remove_file(manifest_path);
    }

    #[tokio::test]
    async fn test_cli_tonight() {
        use crate::commands::tonight::tonight_cmd;
        use arcane_core::db::category::add_category;
        use arcane_core::models::category::Category;

        let pool = initialize_app("sqlite::memory:").await.unwrap();

        add_category(
            Category {
                id: 0,
                name: "Work".to_string(),
                default_minutes: 25,
                color: "red".to_string(),
                is_archived: false,
            },
            &pool,
        )
        .await
        .unwrap();

        // Call tonight_cmd
        tonight_cmd(&pool, "19:00", "Work").await;
        tonight_cmd(&pool, "21:00", "rest").await;

        // Verify that they are inserted in the database
        let today = chrono::Local::now().date_naive();
        let list = arcane_core::db::schedule::list_schedule_overrides(today, &pool).await.unwrap();
        assert_eq!(list.len(), 2);
    }
}
