use arcane_core::{db::category::add_category as db_add_category, models::category::Category};
use sqlx::SqlitePool;

pub async fn add_category(pool: &SqlitePool, name: &str, default_minutes: &u32, color: &str) {
    match db_add_category(
        Category {
            id: 0,
            name: name.to_string(),
            default_minutes: *default_minutes,
            color: color.to_string(),
        },
        pool,
    )
    .await
    {
        Ok(_) => {
            println!(
                "Successfully added category: Name: {}, Default Minutes: {}, Color: {}",
                name, default_minutes, color
            );
        }
        Err(e) => {
            eprintln!("Error adding category '{}': {}", name, e);
        }
    }
}
