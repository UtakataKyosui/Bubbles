use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
}

#[derive(Clone)]
pub struct BubbleDb {
    pool: Pool<Sqlite>,
}

impl BubbleDb {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self, DbError> {
         let database_url = format!("sqlite://{}?mode=rwc", path.as_ref().display());
         let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;
            
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        Ok(Self { pool })
    }
    
    pub fn inner(&self) -> &Pool<Sqlite> {
        &self.pool
    }
}
