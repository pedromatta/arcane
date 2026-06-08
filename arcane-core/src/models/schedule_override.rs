use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScheduleOverride {
    pub id: u32,
    pub category_id: Option<u32>,
    pub override_date: NaiveDate,
    pub time_of_day: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use sqlx::SqlitePool;

    #[sqlx::test(
        migrations = "./src/db/migrations",
        fixtures = "../db/fixtures/categories.sql"
    )]
    async fn test_schedule_override_database_integration(pool: SqlitePool) {
        sqlx::query(
            r#"
            INSERT INTO schedule_overrides (category_id, override_date, time_of_day)
            VALUES (1,'2026-06-08', '14:30'),
                   (NULL,'2026-06-09', '10:00')
            "#,
        )
        .execute(&pool)
        .await
        .expect("Failed to insert schedule override data");

        let overrides: ScheduleOverride = sqlx::query_as("SELECT * FROM schedule_overrides")
            .fetch_all(&pool)
            .await
            .expect("Failed to fetch schedule override data");

        assert_eq!(overrides.len(), 2);

        assert_eq!(overrides[0].id, 1);
        assert_eq!(overrides[0].category_id, Some(1));
        assert_eq!(overrides[0].override_date, NaiveDate::from_ymd(2026, 6, 8));
        assert_eq!(overrides[0].time_of_day, "14:30");

        assert_eq!(overrides[1].id, 2);
        assert_eq!(overrides[1].category_id, None);
        assert_eq!(overrides[1].override_date, NaiveDate::from_ymd(2026, 6, 9));
        assert_eq!(overrides[1].time_of_day, "10:00");
    }
}
