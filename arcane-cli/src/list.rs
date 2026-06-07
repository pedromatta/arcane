use arcane_core::db::category::list_categories_async;
use sqlx::SqlitePool;

pub async fn list_categories(pool: &SqlitePool) {
    let categories = list_categories_async(&pool).await;

    if categories.is_empty() {
        println!("No categories found.");
    } else {
        println!("+----+----------------------+-----------------+------------+");
        println!("| ID | Name                 | Default Minutes | Color      |");
        println!("+----+----------------------+-----------------+------------+");
        for category in categories {
            println!(
                // We don't want big names to mess up the table, so we cut it and add "..." at the
                // end if it's too long
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
    };
}
