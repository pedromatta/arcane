use crate::models::category::Category;
use crate::error::ArcaneError;
use sqlx::SqlitePool;

pub async fn add_category(category: Category, pool: &SqlitePool) -> Result<(), ArcaneError> {
    sqlx::query(
        r#"
            INSERT INTO categories (name, default_minutes, color)
            VALUES (?, ?, ?)
        "#,
    )
    .bind(&category.name)
    .bind(category.default_minutes)
    .bind(&category.color)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_categories(pool: &SqlitePool) -> Result<Vec<Category>, ArcaneError> {
    let categories: Vec<Category> = sqlx::query_as("SELECT * FROM categories")
        .fetch_all(pool)
        .await?;
    Ok(categories)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[sqlx::test(migrations = "./src/db/migrations")]
    async fn test_add_category(pool: SqlitePool) {
        add_category(
            Category {
                id: 0,
                name: "Work".to_string(),
                default_minutes: 25,
                color: "#FF0000".to_string(),
            },
            &pool,
        )
        .await
        .unwrap();

        let category: Category = sqlx::query_as("SELECT * FROM categories")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch category data");

        assert_eq!(category.id, 1);
        assert_eq!(category.name, "Work");
        assert_eq!(category.default_minutes, 25);
        assert_eq!(category.color, "#FF0000");
    }

    #[sqlx::test(migrations = "./src/db/migrations")]
    async fn test_list_categories(pool: SqlitePool) {
        add_category(
            Category {
                id: 0,
                name: "Work".to_string(),
                default_minutes: 25,
                color: "#FF0000".to_string(),
            },
            &pool,
        )
        .await
        .unwrap();

        let categories = list_categories(&pool).await.unwrap();

        assert_eq!(categories[0].id, 1);
        assert_eq!(categories[0].name, "Work");
        assert_eq!(categories[0].default_minutes, 25);
        assert_eq!(categories[0].color, "#FF0000");
    }
}
