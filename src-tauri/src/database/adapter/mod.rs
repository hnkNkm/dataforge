use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;

pub mod postgres;
pub mod mysql;
pub mod sqlite;

/// Supported database types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    SQLite,
}

impl DatabaseType {
    pub fn default_port(&self) -> Option<u16> {
        match self {
            DatabaseType::PostgreSQL => Some(5432),
            DatabaseType::MySQL => Some(3306),
            DatabaseType::SQLite => None, // SQLite doesn't use ports
        }
    }

    pub fn requires_host(&self) -> bool {
        match self {
            DatabaseType::PostgreSQL | DatabaseType::MySQL => true,
            DatabaseType::SQLite => false,
        }
    }

    pub fn requires_credentials(&self) -> bool {
        match self {
            DatabaseType::PostgreSQL | DatabaseType::MySQL => true,
            DatabaseType::SQLite => false,
        }
    }
}

/// Connection parameters for any database type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionParams {
    pub database_type: DatabaseType,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ssl_mode: Option<String>,
    pub connection_timeout: Option<u32>,
    pub max_connections: Option<u32>,
    pub additional_params: HashMap<String, String>,
}

impl ConnectionParams {
    /// Create new connection parameters
    pub fn new(database_type: DatabaseType, database: String) -> Self {
        Self {
            database_type,
            host: if database_type.requires_host() {
                Some("localhost".to_string())
            } else {
                None
            },
            port: database_type.default_port(),
            database,
            username: None,
            password: None,
            ssl_mode: None,
            connection_timeout: Some(5),
            max_connections: Some(5),
            additional_params: HashMap::new(),
        }
    }

    /// Validate connection parameters
    pub fn validate(&self) -> Result<(), AppError> {
        // Check required fields based on database type
        if self.database_type.requires_host() && self.host.is_none() {
            return Err(AppError::Validation("Host is required".to_string()));
        }

        if self.database_type.requires_credentials() {
            if self.username.is_none() {
                return Err(AppError::Validation("Username is required".to_string()));
            }
        }

        if self.database.is_empty() {
            return Err(AppError::Validation("Database name is required".to_string()));
        }

        Ok(())
    }
}

/// Query result row
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRow {
    pub columns: Vec<String>,
    pub values: Vec<Option<String>>,
}

/// Query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<QueryRow>,
    pub rows_affected: Option<u64>,
    pub execution_time: Option<u64>, // in milliseconds
}

/// Column information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
}

/// Table information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub schema: Option<String>,
    pub table_type: String, // TABLE, VIEW, etc.
    pub row_count: Option<i64>,
}

/// Database metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetadata {
    pub version: String,
    pub database_name: String,
    pub size: Option<i64>, // in bytes
    pub encoding: Option<String>,
}

/// Common database operations trait
#[async_trait]
pub trait DatabaseAdapter: Send + Sync {
    /// Connect to the database
    async fn connect(&mut self, params: &ConnectionParams) -> Result<(), AppError>;

    /// Disconnect from the database
    async fn disconnect(&mut self) -> Result<(), AppError>;

    /// Test if the connection is alive
    async fn test_connection(&self) -> Result<bool, AppError>;

    /// Execute a query and return results
    async fn execute_query(&self, query: &str) -> Result<QueryResult, AppError>;

    /// Execute a non-query command (INSERT, UPDATE, DELETE)
    async fn execute_command(&self, command: &str) -> Result<u64, AppError>;

    /// Begin a transaction
    async fn begin_transaction(&mut self) -> Result<(), AppError>;

    /// Commit a transaction
    async fn commit_transaction(&mut self) -> Result<(), AppError>;

    /// Rollback a transaction
    async fn rollback_transaction(&mut self) -> Result<(), AppError>;

    /// Get database metadata
    async fn get_metadata(&self) -> Result<DatabaseMetadata, AppError>;

    /// List all tables
    async fn list_tables(&self) -> Result<Vec<TableInfo>, AppError>;

    /// Get table columns
    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<ColumnInfo>, AppError>;

    /// Get the current database name
    async fn current_database(&self) -> Result<String, AppError>;

    /// Get the connection status
    fn is_connected(&self) -> bool;

    /// Get the database type
    fn database_type(&self) -> DatabaseType;
}

/// Factory function to create appropriate adapter
pub fn create_adapter(database_type: DatabaseType) -> Result<Box<dyn DatabaseAdapter>, AppError> {
    match database_type {
        DatabaseType::PostgreSQL => Ok(Box::new(postgres::PostgresAdapter::new())),
        DatabaseType::MySQL => Ok(Box::new(mysql::MySqlAdapter::new())),
        DatabaseType::SQLite => Ok(Box::new(sqlite::SqliteAdapter::new())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_type_defaults() {
        assert_eq!(DatabaseType::PostgreSQL.default_port(), Some(5432));
        assert_eq!(DatabaseType::MySQL.default_port(), Some(3306));
        assert_eq!(DatabaseType::SQLite.default_port(), None);
    }

    #[test]
    fn test_connection_params_validation() {
        let mut params = ConnectionParams::new(DatabaseType::PostgreSQL, "test_db".to_string());

        // Should fail without username
        assert!(params.validate().is_err());

        // Add username
        params.username = Some("user".to_string());
        assert!(params.validate().is_ok());

        // SQLite shouldn't require credentials
        let sqlite_params = ConnectionParams::new(DatabaseType::SQLite, "test.db".to_string());
        assert!(sqlite_params.validate().is_ok());
    }
}