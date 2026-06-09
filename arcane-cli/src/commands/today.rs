use arcane_core::db::schedule::add_today_override;
use sqlx::SqlitePool;

pub async fn today_cmd(pool: &SqlitePool, time: &str, category: &str) {
    match add_today_override(category, time, pool).await {
        Ok(_) => {
            println!(
                "Successfully added override: category '{}' scheduled for today at {}.",
                category, time
            );
        }
        Err(e) => {
            eprintln!("Error adding today override: {}", e);
        }
    }
}
