use arcane_core::db::category::{remove_category, RemovalResult};
use sqlx::SqlitePool;

pub async fn remove_category_cmd(pool: &SqlitePool, name: &str) {
    match remove_category(name, pool).await {
        Ok(RemovalResult::Deleted) => {
            println!("Category '{}' was successfully removed.", name);
        }
        Ok(RemovalResult::Archived) => {
            println!("Category '{}' has historical sessions and was archived to protect logs.", name);
        }
        Err(e) => {
            eprintln!("Error removing category '{}': {}", name, e);
        }
    }
}
