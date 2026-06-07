use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    id: u32,
    category_id: u32,
    start_time: NaiveDateTime,
    duration_minutes: u32,
    notes: Option<String>,
    rating: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use sqlx::SqlitePool;

    #[sqlx::test(migrations = "./src/db/migrations")]
    async fn test_session_database_integration(pool: SqlitePool) {
        sqlx::query(
            r#"
                INSERT INTO sessions (category_id, start_time, duration_minutes, notes, rating)
                VALUES (1, '2026-06-07 09:00:00', 45, 'Morning work session', 4)
            "#,
        )
        .execute(&pool)
        .await
        .expect("Failed to insert session data");

        let session: Session = sqlx::query_as("SELECT * FROM sessions")
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch session data");

        assert_eq!(session.id, 1);
        assert_eq!(session.category_id, 1);
        assert_eq!(
            session.start_time,
            NaiveDate::from_ymd_opt(2026, 6, 7)
                .unwrap()
                .and_hms_opt(9, 0, 0)
                .unwrap()
        );
        assert_eq!(session.duration_minutes, 45);
        assert_eq!(session.notes, Some("Morning work session".to_string()));
        assert_eq!(session.rating, Some(4));
    }
}
