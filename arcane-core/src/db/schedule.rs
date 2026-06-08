use crate::models::schedule_slot::{ScheduleSlot, ScheduleSlotDetail};
use crate::models::schedule_override::ScheduleOverride;
use crate::error::ArcaneError;
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
    // 1. Resolve category name to ID
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

    // 2. Validate time_of_day format (HH:MM)
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

    // 3. Parse and validate days bitmask
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
}
