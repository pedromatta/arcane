use crate::models::schedule_slot::ScheduleSlot;
use crate::models::schedule_override::ScheduleOverride;
use crate::error::ArcaneError;
use sqlx::SqlitePool;
use chrono::NaiveDate;

pub async fn add_schedule_slot(
    category_id: u32,
    time_of_day: &str,
    days_of_week: u8,
    pool: &SqlitePool,
) -> Result<(), ArcaneError> {
    // Validate time_of_day format (HH:MM)
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

    // Validate days_of_week bitmask
    if days_of_week > 127 {
        return Err(ArcaneError::CategoryValidation(format!(
            "Days bitmask {} exceeds maximum 127",
            days_of_week
        )));
    }

    // Verify category exists and is not archived
    let exists: Option<bool> = sqlx::query_scalar("SELECT is_archived FROM categories WHERE id = ?")
        .bind(category_id)
        .fetch_optional(pool)
        .await?;
    match exists {
        Some(true) => {
            return Err(ArcaneError::CategoryValidation(
                "Cannot schedule slot for an archived category".to_string(),
            ))
        }
        None => {
            return Err(ArcaneError::CategoryValidation(format!(
                "Category ID {} does not exist",
                category_id
            )))
        }
        _ => {}
    }

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
    // Validate category if present
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

    // Validate time format
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
