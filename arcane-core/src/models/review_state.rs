use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReviewState {
    id: u32,
    category_id: u32,
    topic: String,
    #[serde(default = "default_ease_factor")]
    ease_factor: Option<f32>,
    #[serde(default = "default_interval_days")]
    interval_days: Option<u32>,
    next_review_date: NaiveDate,
}

fn default_ease_factor() -> Option<f32> {
    Some(2.5)
}

fn default_interval_days() -> Option<u32> {
    Some(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use sqlx::SqlitePool;

    #[sqlx::test(
        migrations = "./src/db/migrations",
        fixtures("../db/fixtures/categories.sql")
    )]
    async fn test_review_state_database_integration(pool: SqlitePool) {
        sqlx::query(
            r#"
                INSERT INTO review_states (category_id, topic, ease_factor, interval_days, next_review_date)
                VALUES (1, 'Work', 2.5, 3, '2026-06-10')
            "#,
        )
        .execute(&pool)
        .await
        .expect("Failed to insert review state data");

        let review_state: ReviewState = sqlx::query_as("SELECT * FROM review_states")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch review state data");

        assert_eq!(review_state.id, 1);
        assert_eq!(review_state.category_id, 1);
        assert_eq!(review_state.topic, "Work".to_string());
        assert_eq!(review_state.ease_factor, Some(2.5));
        assert_eq!(review_state.interval_days, Some(3));
        assert_eq!(
            review_state.next_review_date,
            NaiveDate::from_ymd_opt(2026, 6, 10).unwrap()
        );
    }
}
