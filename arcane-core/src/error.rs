use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArcaneError {
    #[error("Database error occurred: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Database migration failed: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Failed to parse configuration: {0}")]
    Config(#[from] toml::de::Error),

    #[error("I/O error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration validation failed: {0}")]
    ConfigValidation(String),

    #[error("Failed to parse structure date: {0}")]
    Chrono(#[from] chrono::ParseError),

    #[error("Category validation failed: {0}")]
    CategoryValidation(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Error as SqlxError;

    #[test]
    fn formats_database_error_correctly() {
        let sqlx_err = SqlxError::RowNotFound;
        let app_err: ArcaneError = sqlx_err.into();
        assert_eq!(
            app_err.to_string(),
            "Database error occurred: no rows returned by a query that expected to return at least one row"
        );
    }
}
