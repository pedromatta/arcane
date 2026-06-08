use crate::models::category::Category;
use crate::error::ArcaneError;
use sqlx::SqlitePool;

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
    let categories: Vec<Category> = sqlx::query_as("SELECT * FROM categories")
        .fetch_all(pool)
        .await?;
    Ok(categories)
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
}
