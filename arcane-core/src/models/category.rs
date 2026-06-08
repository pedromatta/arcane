use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: u32,
    pub name: String,
    pub default_minutes: u32,
    pub color: String,
    #[serde(default)]
    pub is_archived: bool,
}
