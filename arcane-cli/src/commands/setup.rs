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
        .or_else(get_default_config_path)
        .context("Could not determine configuration path")?;

    let config_content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file at {:?}", config_path))?;

    toml::from_str(&config_content).context("Failed to parse config file as TOML")
}

pub async fn run_setup(pool: &SqlitePool) -> Result<()> {
    println!("Welcome to the Arcane setup wizard!");
    println!("We will configure your settings, categories, and schedule slots.\n");

    let notifications_enabled = dialoguer::Confirm::new()
        .with_prompt("Enable desktop notifications?")
        .default(true)
        .interact()?;

    // Update config.toml
    if let Some(config_path) = get_default_config_path() {
        let current_db_path = if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str::<AppConfig>(&content) {
                config.general.database_path
            } else {
                None
            }
        } else {
            None
        };

        let db_path_str = current_db_path.unwrap_or_else(|| "arcane.db".to_string());

        let new_content = format!(
            "[general]\ndatabase_path = \"{}\"\nnotifications_enabled = {}\n",
            db_path_str, notifications_enabled
        );

        if let Err(e) = std::fs::write(&config_path, new_content) {
            eprintln!("Warning: Failed to update config.toml: {}", e);
        } else {
            println!("Saved general settings to {:?}", config_path);
        }
    }

    println!("\n--- Step 1: Define Categories ---");
    loop {
        let name: String = dialoguer::Input::new()
            .with_prompt("Category Name (e.g., Rust, Exercise)")
            .validate_with(|input: &String| {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    Err("Category name cannot be empty".to_string())
                } else {
                    Ok(())
                }
            })
            .interact_text()?;

        let default_minutes: u32 = dialoguer::Input::new()
            .with_prompt("Default duration in minutes")
            .default(25)
            .interact_text()?;

        let colors = vec![
            "white",
            "red",
            "green",
            "yellow",
            "blue",
            "magenta",
            "cyan",
            "gray",
            "darkgray",
            "lightred",
            "lightgreen",
            "lightyellow",
            "lightblue",
            "lightmagenta",
            "lightcyan",
            "black",
            "Custom HEX",
        ];

        let color_index = dialoguer::Select::new()
            .with_prompt("Select category color")
            .items(&colors)
            .default(0)
            .interact()?;

        let color = if colors[color_index] == "Custom HEX" {
            dialoguer::Input::<String>::new()
                .with_prompt("Enter custom color (named, ANSI integer, or HEX)")
                .validate_with(|input: &String| {
                    match arcane_core::db::category::validate_color(input) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e.to_string()),
                    }
                })
                .interact_text()?
        } else {
            colors[color_index].to_string()
        };

        let category = arcane_core::models::category::Category {
            id: 0,
            name: name.clone(),
            default_minutes,
            color,
            is_archived: false,
        };

        match arcane_core::db::category::add_category(category, pool).await {
            Ok(_) => println!("Category '{}' added successfully.", name),
            Err(e) => eprintln!("Failed to add category: {}", e),
        }

        let add_another = dialoguer::Confirm::new()
            .with_prompt("Would you like to add another category?")
            .default(false)
            .interact()?;

        if !add_another {
            break;
        }
    }

    println!("\n--- Step 2: Configure Weekly Schedule Slots ---");
    let setup_slots = dialoguer::Confirm::new()
        .with_prompt("Would you like to schedule any weekly slots now?")
        .default(true)
        .interact()?;

    if setup_slots {
        loop {
            let categories = match arcane_core::db::category::list_categories(pool).await {
                Ok(cats) => cats,
                Err(e) => {
                    eprintln!("Error loading categories: {}", e);
                    break;
                }
            };

            if categories.is_empty() {
                println!("No active categories found. Please define categories first.");
                break;
            }

            let cat_names: Vec<String> = categories.iter().map(|c| c.name.clone()).collect();
            let cat_index = dialoguer::Select::new()
                .with_prompt("Select Category")
                .items(&cat_names)
                .default(0)
                .interact()?;

            let selected_category = &cat_names[cat_index];

            let time: String = dialoguer::Input::new()
                .with_prompt("Start time (HH:MM)")
                .validate_with(|input: &String| {
                    let parts: Vec<&str> = input.split(':').collect();
                    if parts.len() != 2 {
                        return Err("Use HH:MM format".to_string());
                    }
                    let hour: Result<u32, _> = parts[0].parse();
                    let min: Result<u32, _> = parts[1].parse();
                    match (hour, min) {
                        (Ok(h), Ok(m)) if h <= 23 && m <= 59 => Ok(()),
                        _ => Err("Invalid hours/minutes. Must be 00-23 and 00-59.".to_string()),
                    }
                })
                .interact_text()?;

            let days: String = dialoguer::Input::new()
                .with_prompt("Weekdays (e.g. mon,tue or weekdays or everyday)")
                .validate_with(|input: &String| {
                    match arcane_core::db::schedule::parse_weekdays(input) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e.to_string()),
                    }
                })
                .interact_text()?;

            match arcane_core::db::schedule::add_schedule_slot(
                selected_category,
                &time,
                &days,
                pool,
            )
            .await
            {
                Ok(_) => println!("Scheduled weekly slot successfully."),
                Err(e) => eprintln!("Failed to schedule slot: {}", e),
            }

            let add_another_slot = dialoguer::Confirm::new()
                .with_prompt("Would you like to schedule another weekly slot?")
                .default(false)
                .interact()?;

            if !add_another_slot {
                break;
            }
        }
    }

    println!("\nSetup complete! Welcome to Arcane.");
    Ok(())
}
