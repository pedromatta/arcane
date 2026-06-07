use arcane_core::{db::category::add_category_async, models::category::Category};
use sqlx::SqlitePool;

pub async fn add_category(pool: &SqlitePool, name: &str, default_minutes: &u32, color: &str) {
    add_category_async(
        Category {
            id: 0,
            name: name.to_string(),
            default_minutes: *default_minutes,
            color: color.to_string(),
        },
        &pool,
    )
    .await;
    println!(
        "Adding category: Name: {}, Default Minutes: {}, Color: {}",
        name, default_minutes, color
    );
}
