use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReviewState {
    id: u32,
    category_id: u32,
    topic: String,
    ease_factor: f32,
    interval_days: u32,
    next_review_date: String,
}

impl ReviewState {
    pub fn is_due(&self, current_date: NaiveDate) -> bool {
        self.get_date() <= current_date
    }

    fn get_date(&self) -> NaiveDate {
        NaiveDate::parse_from_str(&self.next_review_date, "%Y-%m-%d")
            .expect("Database next_review_date column contained invalid date format")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_review() -> ReviewState {
        ReviewState {
            id: 1,
            category_id: 1,
            topic: "Work".to_string(),
            ease_factor: 2.5,
            interval_days: 1,
            next_review_date: "2026-06-06".to_string(),
        }
    }

    #[test]
    fn test_review_state_creation() {
        let review = mock_review();

        assert_eq!(review.id, 1);
        assert_eq!(review.category_id, 1);
        assert_eq!(review.topic, "Work".to_string());
        assert_eq!(review.ease_factor, 2.5);
        assert_eq!(review.interval_days, 1);
        assert_eq!(review.next_review_date, "2026-06-06".to_string());
    }

    #[test]
    fn test_review_is_due() {
        let review = mock_review();

        let past_date = NaiveDate::from_ymd_opt(2026, 6, 5).unwrap();
        let exact_date = NaiveDate::from_ymd_opt(2026, 6, 6).unwrap();
        let future_date = NaiveDate::from_ymd_opt(2026, 6, 7).unwrap();

        assert!(!review.is_due(past_date));
        assert!(review.is_due(exact_date));
        assert!(review.is_due(future_date));
    }
}
