use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScheduleSlot {
    pub id: u32,
    pub category_id: u32,
    pub time_of_day: String,
    pub days_of_week: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct ScheduleSlotDetail {
    pub id: u32,
    pub category_id: u32,
    pub category_name: String,
    pub time_of_day: String,
    pub days_of_week: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[sqlx::test(
        migrations = "./src/db/migrations",
        fixtures("../db/fixtures/categories.sql")
    )]
    async fn test_schedule_slot_database_integration(pool: SqlitePool) {
        sqlx::query(
            r#"
            INSERT INTO schedule_slots (category_id, time_of_day, days_of_week)
            VALUES (1,'09:00', 31)
            "#,
        )
        .execute(&pool)
        .await
        .expect("Failed to insert schedule slot data");

        let slot: ScheduleSlot = sqlx::query_as("SELECT * FROM schedule_slots")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch schedule slot data");

        assert_eq!(slot.id, 1);
        assert_eq!(slot.category_id, 1);
        assert_eq!(slot.time_of_day, "09:00");
        assert_eq!(slot.days_of_week, 31);
    }
}
