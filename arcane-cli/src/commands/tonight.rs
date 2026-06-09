use arcane_core::db::schedule::add_tonight_override;
use sqlx::SqlitePool;

pub async fn tonight_cmd(pool: &SqlitePool, time: &str, category: &str) {
    match add_tonight_override(category, time, pool).await {
        Ok(_) => {
            println!(
                "Successfully added override: category '{}' scheduled for tonight at {}.",
                category, time
            );
        }
        Err(e) => {
            eprintln!("Error adding tonight override: {}", e);
        }
    }
}
