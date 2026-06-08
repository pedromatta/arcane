use crate::models::schedule_slot::{ScheduleSlot, ScheduleSlotDetail};
use crate::models::schedule_override::ScheduleOverride;
use crate::error::ArcaneError;
use crate::config::ImportManifest;
use sqlx::SqlitePool;
use chrono::NaiveDate;

pub fn parse_weekdays(input: &str) -> Result<u8, ArcaneError> {
    let trimmed = input.trim().to_lowercase();
    if trimmed.is_empty() {
        return Err(ArcaneError::CategoryValidation("Weekdays input cannot be empty".to_string()));
    }

    match trimmed.as_str() {
        "everyday" | "all" | "any" | "daily" => return Ok(127),
        "weekdays" | "weekday" | "workdays" => return Ok(31),
        "weekend" | "weekends" => return Ok(96),
        _ => {}
    }

    let mut bitmask: u8 = 0;
    for part in trimmed.split(',') {
        let token = part.trim();
        if token.is_empty() {
            continue;
        }
        let bit = match token {
            "mon" | "monday" => 1,
            "tue" | "tuesday" => 2,
            "wed" | "wednesday" => 4,
            "thu" | "thursday" => 8,
            "fri" | "friday" => 16,
            "sat" | "saturday" => 32,
            "sun" | "sunday" => 64,
            _ => {
                return Err(ArcaneError::CategoryValidation(format!(
                    "Invalid weekday token '{}'. Valid options: mon, tue, wed, thu, fri, sat, sun, weekdays, weekend, everyday",
                    token
                )))
            }
        };
        bitmask |= bit;
    }

    if bitmask == 0 {
        return Err(ArcaneError::CategoryValidation("No valid weekdays parsed from input".to_string()));
    }

    Ok(bitmask)
}

pub async fn add_schedule_slot(
    category_name: &str,
    time_of_day: &str,
    days_of_week_str: &str,
    pool: &SqlitePool,
) -> Result<(), ArcaneError> {
    let category_id_opt: Option<i64> = sqlx::query_scalar("SELECT id FROM categories WHERE name = ? AND is_archived = 0")
        .bind(category_name.trim())
        .fetch_optional(pool)
        .await?;

    let category_id = match category_id_opt {
        Some(id) => id as u32,
        None => {
            return Err(ArcaneError::CategoryValidation(format!(
                "Category '{}' does not exist or is archived",
                category_name
            )))
        }
    };

    let parts: Vec<&str> = time_of_day.split(':').collect();
    if parts.len() != 2 {
        return Err(ArcaneError::CategoryValidation(format!(
            "Invalid time format '{}'. Use HH:MM.",
            time_of_day
        )));
    }
    let hour: u32 = parts[0].parse().map_err(|_| {
        ArcaneError::CategoryValidation(format!("Invalid hour in time '{}'", time_of_day))
    })?;
    let min: u32 = parts[1].parse().map_err(|_| {
        ArcaneError::CategoryValidation(format!("Invalid minute in time '{}'", time_of_day))
    })?;
    if hour > 23 || min > 59 {
        return Err(ArcaneError::CategoryValidation(format!(
            "Time parameters out of bounds in '{}'",
            time_of_day
        )));
    }

    let days_of_week = parse_weekdays(days_of_week_str)?;

    sqlx::query(
        r#"
            INSERT INTO schedule_slots (category_id, time_of_day, days_of_week)
            VALUES (?, ?, ?)
        "#,
    )
    .bind(category_id)
    .bind(time_of_day)
    .bind(days_of_week)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_schedule_slots(pool: &SqlitePool) -> Result<Vec<ScheduleSlot>, ArcaneError> {
    let slots: Vec<ScheduleSlot> = sqlx::query_as("SELECT * FROM schedule_slots")
        .fetch_all(pool)
        .await?;
    Ok(slots)
}

pub async fn list_schedule_slots_detail(pool: &SqlitePool) -> Result<Vec<ScheduleSlotDetail>, ArcaneError> {
    let slots: Vec<ScheduleSlotDetail> = sqlx::query_as(
        r#"
            SELECT s.id, s.category_id, c.name as category_name, s.time_of_day, s.days_of_week
            FROM schedule_slots s
            JOIN categories c ON s.category_id = c.id
            WHERE c.is_archived = 0
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(slots)
}

pub async fn remove_schedule_slot(slot_id: u32, pool: &SqlitePool) -> Result<(), ArcaneError> {
    let affected = sqlx::query("DELETE FROM schedule_slots WHERE id = ?")
        .bind(slot_id)
        .execute(pool)
        .await?
        .rows_affected();

    if affected == 0 {
        return Err(ArcaneError::CategoryValidation(format!(
            "Schedule slot ID {} does not exist",
            slot_id
        )));
    }

    Ok(())
}

pub async fn add_schedule_override(
    category_id: Option<u32>,
    override_date: NaiveDate,
    time_of_day: &str,
    pool: &SqlitePool,
) -> Result<(), ArcaneError> {
    if let Some(cat_id) = category_id {
        let exists: Option<bool> = sqlx::query_scalar("SELECT is_archived FROM categories WHERE id = ?")
            .bind(cat_id)
            .fetch_optional(pool)
            .await?;
        match exists {
            Some(true) => {
                return Err(ArcaneError::CategoryValidation(
                    "Cannot schedule override for an archived category".to_string(),
                ))
            }
            None => {
                return Err(ArcaneError::CategoryValidation(format!(
                    "Category ID {} does not exist",
                    cat_id
                )))
            }
            _ => {}
        }
    }

    let parts: Vec<&str> = time_of_day.split(':').collect();
    if parts.len() != 2 {
        return Err(ArcaneError::CategoryValidation(format!(
            "Invalid time format '{}'. Use HH:MM.",
            time_of_day
        )));
    }

    sqlx::query(
        r#"
            INSERT OR REPLACE INTO schedule_overrides (category_id, override_date, time_of_day)
            VALUES (?, ?, ?)
        "#,
    )
    .bind(category_id)
    .bind(override_date)
    .bind(time_of_day)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_schedule_overrides(
    date: NaiveDate,
    pool: &SqlitePool,
) -> Result<Vec<ScheduleOverride>, ArcaneError> {
    let overrides: Vec<ScheduleOverride> =
        sqlx::query_as("SELECT * FROM schedule_overrides WHERE override_date = ?")
            .bind(date)
            .fetch_all(pool)
            .await?;
    Ok(overrides)
}

pub async fn import_manifest(manifest: &ImportManifest, pool: &SqlitePool) -> Result<(), ArcaneError> {
    let mut tx = pool.begin().await?;

    // 1. Reconcile categories
    for cat in &manifest.categories {
        let trimmed_name = cat.name.trim();
        if trimmed_name.is_empty() {
            return Err(ArcaneError::CategoryValidation("Category name cannot be empty".to_string()));
        }
        
        validate_color(&cat.color)?;

        let existing_opt: Option<(i64, bool)> = sqlx::query_as(
            "SELECT id, is_archived FROM categories WHERE name = ?"
        )
        .bind(trimmed_name)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some((id, is_archived)) = existing_opt {
            if is_archived {
                sqlx::query("UPDATE categories SET default_minutes = ?, color = ?, is_archived = 0 WHERE id = ?")
                    .bind(cat.default_minutes)
                    .bind(&cat.color)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            } else {
                sqlx::query("UPDATE categories SET default_minutes = ?, color = ? WHERE id = ?")
                    .bind(cat.default_minutes)
                    .bind(&cat.color)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
        } else {
            sqlx::query("INSERT INTO categories (name, default_minutes, color, is_archived) VALUES (?, ?, ?, 0)")
                .bind(trimmed_name)
                .bind(cat.default_minutes)
                .bind(&cat.color)
                .execute(&mut *tx)
                .await?;
        }
    }

    // 2. Clear old schedule slots if new ones are provided
    if manifest.schedule.is_some() {
        sqlx::query("DELETE FROM schedule_slots")
            .execute(&mut *tx)
            .await?;
    }

    // 3. Reconcile schedule slots
    if let Some(ref schedule_list) = manifest.schedule {
        for sched in schedule_list {
            let category_id_opt: Option<i64> = sqlx::query_scalar("SELECT id FROM categories WHERE name = ? AND is_archived = 0")
                .bind(sched.category.trim())
                .fetch_optional(&mut *tx)
                .await?;

            let category_id = match category_id_opt {
                Some(id) => id as u32,
                None => {
                    return Err(ArcaneError::CategoryValidation(format!(
                        "Category '{}' does not exist in manifest or active database",
                        sched.category
                    )))
                }
            };

            let parts: Vec<&str> = sched.time.split(':').collect();
            if parts.len() != 2 {
                return Err(ArcaneError::CategoryValidation(format!("Invalid time format '{}'. Use HH:MM.", sched.time)));
            }
            let hour: u32 = parts[0].parse().map_err(|_| ArcaneError::CategoryValidation(format!("Invalid hour in time '{}'", sched.time)))?;
            let min: u32 = parts[1].parse().map_err(|_| ArcaneError::CategoryValidation(format!("Invalid minute in time '{}'", sched.time)))?;
            if hour > 23 || min > 59 {
                return Err(ArcaneError::CategoryValidation(format!("Time parameters out of bounds in '{}'", sched.time)));
            }

            if sched.days > 127 {
                return Err(ArcaneError::CategoryValidation(format!("Days bitmask {} exceeds 127", sched.days)));
            }

            sqlx::query("INSERT INTO schedule_slots (category_id, time_of_day, days_of_week) VALUES (?, ?, ?)")
                .bind(category_id)
                .bind(&sched.time)
                .bind(sched.days)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;
    Ok(())
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
    use crate::models::category::Category;
    use crate::db::category::add_category;

    #[test]
    fn test_parse_weekdays() {
        assert_eq!(parse_weekdays("mon").unwrap(), 1);
        assert_eq!(parse_weekdays("Mon, Tue").unwrap(), 3);
        assert_eq!(parse_weekdays("weekdays").unwrap(), 31);
        assert_eq!(parse_weekdays("weekend").unwrap(), 96);
        assert_eq!(parse_weekdays("everyday").unwrap(), 127);
        assert!(parse_weekdays("invalid").is_err());
    }

    #[sqlx::test(migrations = "./src/db/migrations")]
    async fn test_schedule_db_operations(pool: SqlitePool) {
        add_category(
            Category {
                id: 0,
                name: "Work".to_string(),
                default_minutes: 25,
                color: "red".to_string(),
                is_archived: false,
            },
            &pool,
        )
        .await
        .unwrap();

        add_schedule_slot("Work", "09:00", "mon,tue", &pool).await.unwrap();

        let details = list_schedule_slots_detail(&pool).await.unwrap();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].category_name, "Work");
        assert_eq!(details[0].time_of_day, "09:00");
        assert_eq!(details[0].days_of_week, 3);
    }

    #[sqlx::test(migrations = "./src/db/migrations")]
    async fn test_import_manifest(pool: SqlitePool) {
        use crate::config::{CategoryConfig, ScheduleConfig};

        let manifest = ImportManifest {
            categories: vec![
                CategoryConfig {
                    name: "Work".to_string(),
                    default_minutes: 25,
                    color: "red".to_string(),
                },
                CategoryConfig {
                    name: "Rust".to_string(),
                    default_minutes: 60,
                    color: "magenta".to_string(),
                },
            ],
            schedule: Some(vec![
                ScheduleConfig {
                    time: "09:00".to_string(),
                    category: "Work".to_string(),
                    days: 31,
                },
                ScheduleConfig {
                    time: "11:00".to_string(),
                    category: "Rust".to_string(),
                    days: 127,
                },
            ]),
        };

        import_manifest(&manifest, &pool).await.unwrap();

        let cats = crate::db::category::list_categories(&pool).await.unwrap();
        assert_eq!(cats.len(), 2);
        assert_eq!(cats[0].name, "Work");
        assert_eq!(cats[1].name, "Rust");

        let slots = list_schedule_slots_detail(&pool).await.unwrap();
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].category_name, "Work");
        assert_eq!(slots[0].time_of_day, "09:00");
        assert_eq!(slots[0].days_of_week, 31);
    }
}
