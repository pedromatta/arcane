use arcane_core::db::category::list_categories as db_list_categories;
use sqlx::SqlitePool;

pub async fn list_categories(pool: &SqlitePool) {
    match db_list_categories(pool).await {
        Ok(categories) => {
            if categories.is_empty() {
                println!("No categories found.");
            } else {
                println!("+----+----------------------+-----------------+------------+");
                println!("| ID | Name                 | Default Minutes | Color      |");
                println!("+----+----------------------+-----------------+------------+");
                for category in categories {
                    println!(
                        "| {:<2} | {:<20} | {:<15} | {:<10} |",
                        category.id,
                        if category.name.len() > 20 {
                            format!("{}...", &category.name[..17])
                        } else {
                            category.name.clone()
                        },
                        category.default_minutes,
                        if category.color.len() > 10 {
                            format!("{}...", &category.color[..7])
                        } else {
                            category.color.clone()
                        }
                    );
                }
                println!("+----+----------------------+-----------------+------------+");
            }
        }
        Err(e) => {
            eprintln!("Error listing categories: {}", e);
        }
    }
}
