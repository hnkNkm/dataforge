use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;
use std::time::Duration;

use super::{DatabaseConfig, DatabaseError, Result};

#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    pool: PgPool,
}

impl DatabaseConnection {
    /// Create a new database connection pool
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections.unwrap_or(5))
            .acquire_timeout(Duration::from_secs(5))
            .connect(&config.connection_string())
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Create a connection from DATABASE_URL environment variable
    pub async fn from_env() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|e| DatabaseError::ConfigError(format!("DATABASE_URL not set: {}", e)))?;

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&database_url)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Test the connection with a simple query
    pub async fn test_connection(&self) -> Result<bool> {
        let result: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&self.pool)
            .await?;

        Ok(result.0 == 1)
    }

    /// Get the current database name
    pub async fn current_database(&self) -> Result<String> {
        let row = sqlx::query("SELECT current_database()")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get::<String, _>(0))
    }

    /// Get PostgreSQL version
    pub async fn version(&self) -> Result<String> {
        let row = sqlx::query("SELECT version()")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get::<String, _>(0))
    }

    /// List all tables in the current database
    pub async fn list_tables(&self) -> Result<Vec<String>> {
        let rows = sqlx::query(
            r#"
            SELECT tablename
            FROM pg_tables
            WHERE schemaname = 'public'
            ORDER BY tablename
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|row| row.get::<String, _>(0)).collect())
    }

    /// Close the connection pool
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Get a reference to the pool for advanced operations
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    #[tokio::test]
    async fn test_postgres_connection() {
        // Load .env file for testing
        dotenv().ok();

        // Skip test if DATABASE_URL is not set
        if std::env::var("DATABASE_URL").is_err() {
            println!("Skipping test: DATABASE_URL not set");
            return;
        }

        // Create connection from environment
        let conn = DatabaseConnection::from_env().await;
        assert!(conn.is_ok(), "Failed to create connection: {:?}", conn.err());

        let conn = conn.unwrap();

        // Test connection
        let test_result = conn.test_connection().await;
        assert!(test_result.is_ok(), "Connection test failed: {:?}", test_result.err());
        assert!(test_result.unwrap(), "Connection test returned false");

        // Get database name
        let db_name = conn.current_database().await;
        assert!(db_name.is_ok(), "Failed to get database name: {:?}", db_name.err());
        println!("Connected to database: {}", db_name.unwrap());

        // Get PostgreSQL version
        let version = conn.version().await;
        assert!(version.is_ok(), "Failed to get version: {:?}", version.err());
        println!("PostgreSQL version: {}", version.unwrap());

        // List tables
        let tables = conn.list_tables().await;
        assert!(tables.is_ok(), "Failed to list tables: {:?}", tables.err());
        println!("Tables in database: {:?}", tables.unwrap());

        // Close connection
        conn.close().await;
    }

    #[tokio::test]
    async fn test_connection_with_config() {
        dotenv().ok();

        // Skip test if environment variables are not set
        if std::env::var("DB_NAME").is_err() {
            println!("Skipping test: DB_NAME not set");
            return;
        }

        // Create config from environment
        let config = DatabaseConfig::from_env();
        assert!(config.is_ok(), "Failed to create config: {:?}", config.err());

        let config = config.unwrap();
        println!("Database config: {:?}", config);

        // Create connection with config
        let conn = DatabaseConnection::new(&config).await;
        assert!(conn.is_ok(), "Failed to create connection: {:?}", conn.err());

        let conn = conn.unwrap();

        // Test connection
        let test_result = conn.test_connection().await;
        assert!(test_result.is_ok() && test_result.unwrap());

        conn.close().await;
    }
}