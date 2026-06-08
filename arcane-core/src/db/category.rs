use crate::models::category::Category;
use crate::error::ArcaneError;
use sqlx::SqlitePool;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RemovalResult {
    Deleted,
    Archived,
}

pub async fn add_category(category: Category, pool: &SqlitePool) -> Result<(), ArcaneError> {
    let trimmed_name = category.name.trim();
    if trimmed_name.is_empty() {
        return Err(ArcaneError::CategoryValidation("Category name cannot be empty".to_string()));
    }

    // Uniqueness validation
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM categories WHERE name = ?)")
        .bind(trimmed_name)
        .fetch_one(pool)
        .await?;
    if exists {
        return Err(ArcaneError::CategoryValidation(format!(
            "Category name '{}' already exists",
            trimmed_name
        )));
    }

    // Color validation
    validate_color(&category.color)?;

    sqlx::query(
        r#"
            INSERT INTO categories (name, default_minutes, color, is_archived)
            VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(trimmed_name)
    .bind(category.default_minutes)
    .bind(&category.color)
    .bind(category.is_archived)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_categories(pool: &SqlitePool) -> Result<Vec<Category>, ArcaneError> {
    let categories: Vec<Category> = sqlx::query_as("SELECT * FROM categories WHERE is_archived = 0")
        .fetch_all(pool)
        .await?;
    Ok(categories)
}

pub async fn remove_category(name: &str, pool: &SqlitePool) -> Result<RemovalResult, ArcaneError> {
    let trimmed_name = name.trim();

    let category_id_opt: Option<i64> = sqlx::query_scalar("SELECT id FROM categories WHERE name = ?")
        .bind(trimmed_name)
        .fetch_optional(pool)
        .await?;

    let category_id = match category_id_opt {
        Some(id) => id,
        None => return Err(ArcaneError::CategoryValidation(format!("Category '{}' does not exist", trimmed_name))),
    };

    let session_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions WHERE category_id = ?")
        .bind(category_id)
        .fetch_one(pool)
        .await?;

    if session_count == 0 {
        sqlx::query("DELETE FROM categories WHERE id = ?")
            .bind(category_id)
            .execute(pool)
            .await?;
        Ok(RemovalResult::Deleted)
    } else {
        sqlx::query("UPDATE categories SET is_archived = 1 WHERE id = ?")
            .bind(category_id)
            .execute(pool)
            .await?;
        Ok(RemovalResult::Archived)
    }
}

fn validate_color(color: &str) -> Result<(), ArcaneError> {
    let trimmed = color.trim().to_lowercase();
    if trimmed.is_empty() {
        return Err(ArcaneError::CategoryValidation("Color cannot be empty".to_string()));
    }

    const NAMED_COLORS: &[&str] = &[
        "black", "red", "green", "yellow", "blue", "magenta", "cyan", "gray",
        "darkgray", "lightred", "lightgreen", "lightyellow", "lightblue",
        "lightmagenta", "lightcyan", "white",
    ];

    if NAMED_COLORS.contains(&trimmed.as_str()) {
        return Ok(());
    }

    if let Ok(_ansi_val) = trimmed.parse::<u8>() {
        return Ok(());
    }

    let hex_body = if trimmed.starts_with('#') {
        &trimmed[1..]
    } else {
        &trimmed[..]
    };

    if (hex_body.len() == 3 || hex_body.len() == 6) && hex_body.chars().all(|c| c.is_ascii_hexdigit()) {
        return Ok(());
    }

    Err(ArcaneError::CategoryValidation(format!(
        "Color must be a valid HEX code (with or without '#'), an ANSI integer (0-255), or one of the named colors: {:?}",
        NAMED_COLORS
    )))
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
                is_archived: false,
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
        assert!(!category.is_archived);
    }

    #[sqlx::test(migrations = "./src/db/migrations")]
    async fn test_category_validation(pool: SqlitePool) {
        // Test 1: Empty name
        let empty_name_cat = Category {
            id: 0,
            name: "   ".to_string(),
            default_minutes: 25,
            color: "red".to_string(),
            is_archived: false,
        };
        let res = add_category(empty_name_cat, &pool).await;
        assert!(matches!(res, Err(ArcaneError::CategoryValidation(_))));

        // Test 2: Valid insertion
        let valid_cat = Category {
            id: 0,
            name: "Work".to_string(),
            default_minutes: 25,
            color: "red".to_string(),
            is_archived: false,
        };
        assert!(add_category(valid_cat, &pool).await.is_ok());

        // Test 3: Duplicate name
        let dup_cat = Category {
            id: 0,
            name: "Work".to_string(),
            default_minutes: 30,
            color: "blue".to_string(),
            is_archived: false,
        };
        let res = add_category(dup_cat, &pool).await;
        assert!(matches!(res, Err(ArcaneError::CategoryValidation(_))));

        // Test 4: Valid HEX colors
        let valid_hex_1 = Category {
            id: 0,
            name: "Hex1".to_string(),
            default_minutes: 25,
            color: "#FFAA00".to_string(),
            is_archived: false,
        };
        assert!(add_category(valid_hex_1, &pool).await.is_ok());

        let valid_hex_2 = Category {
            id: 0,
            name: "Hex2".to_string(),
            default_minutes: 25,
            color: "abc".to_string(),
            is_archived: false,
        };
        assert!(add_category(valid_hex_2, &pool).await.is_ok());

        // Test 5: Valid ANSI color
        let valid_ansi = Category {
            id: 0,
            name: "Ansi".to_string(),
            default_minutes: 25,
            color: "128".to_string(),
            is_archived: false,
        };
        assert!(add_category(valid_ansi, &pool).await.is_ok());

        // Test 6: Invalid color format
        let invalid_color = Category {
            id: 0,
            name: "InvalidColor".to_string(),
            default_minutes: 25,
            color: "not-a-color".to_string(),
            is_archived: false,
        };
        let res = add_category(invalid_color, &pool).await;
        assert!(matches!(res, Err(ArcaneError::CategoryValidation(_))));
    }

    #[sqlx::test(migrations = "./src/db/migrations")]
    async fn test_list_categories(pool: SqlitePool) {
        add_category(
            Category {
                id: 0,
                name: "Work".to_string(),
                default_minutes: 25,
                color: "#FF0000".to_string(),
                is_archived: false,
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
        assert!(!categories[0].is_archived);
    }

    #[sqlx::test(migrations = "./src/db/migrations")]
    async fn test_remove_category(pool: SqlitePool) {
        add_category(
            Category {
                id: 0,
                name: "Work".to_string(),
                default_minutes: 25,
                color: "#FF0000".to_string(),
                is_archived: false,
            },
            &pool,
        )
        .await
        .unwrap();

        // 1. Remove category that has no sessions (expect RemovalResult::Deleted)
        let res = remove_category("Work", &pool).await.unwrap();
        assert_eq!(res, RemovalResult::Deleted);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM categories WHERE name = 'Work'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);

        // 2. Re-create and simulate a session log
        add_category(
            Category {
                id: 0,
                name: "Rust".to_string(),
                default_minutes: 60,
                color: "magenta".to_string(),
                is_archived: false,
            },
            &pool,
        )
        .await
        .unwrap();

        sqlx::query(
            r#"
            INSERT INTO sessions (category_id, start_time, duration_minutes, notes, rating)
            VALUES (2, '2026-06-08 10:00:00', 60, 'Session', 5)
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // 3. Remove category that has sessions (expect RemovalResult::Archived)
        let res = remove_category("Rust", &pool).await.unwrap();
        assert_eq!(res, RemovalResult::Archived);

        let category: Category = sqlx::query_as("SELECT * FROM categories WHERE name = 'Rust'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(category.is_archived);

        // 4. Removing non-existent category (expect error)
        let err = remove_category("NonExistent", &pool).await;
        assert!(err.is_err());
    }
}
