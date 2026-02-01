use async_trait::async_trait;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteRow};
use sqlx::{Column, Row, TypeInfo};
use std::path::Path;
use std::time::Duration;

use super::{
    ColumnInfo, ConnectionParams, DatabaseAdapter, DatabaseMetadata, DatabaseType, QueryResult,
    QueryRow, TableInfo,
};
use crate::database::dialect::{SqlDialect, SQLiteDialect};
use crate::database::capabilities::{DatabaseCapabilities, QueryTemplates};
use crate::error::AppError;

pub struct SqliteAdapter {
    pool: Option<SqlitePool>,
    connected: bool,
    database_path: String,
    dialect: SQLiteDialect,
}

impl SqliteAdapter {
    pub fn new() -> Self {
        Self {
            pool: None,
            connected: false,
            database_path: String::new(),
            dialect: SQLiteDialect::new(),
        }
    }

    fn get_pool(&self) -> Result<&SqlitePool, AppError> {
        self.pool
            .as_ref()
            .ok_or_else(|| AppError::Database(crate::database::DatabaseError::ConnectionFailed(
                "Not connected to database".to_string(),
            )))
    }

    fn build_connection_string(params: &ConnectionParams) -> Result<String, AppError> {
        // For SQLite, the database parameter is the file path
        let db_path = &params.database;

        // Ensure parent directory exists
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::Database(crate::database::DatabaseError::ConnectionFailed(
                    format!("Failed to create database directory: {}", e),
                ))
            })?;
        }

        // SQLite connection string with create mode
        // If file doesn't exist, SQLite will create it automatically
        Ok(format!("sqlite://{}?mode=rwc", db_path))
    }
}

#[async_trait]
impl DatabaseAdapter for SqliteAdapter {
    async fn connect(&mut self, params: &ConnectionParams) -> Result<(), AppError> {
        params.validate()?;

        let connection_string = Self::build_connection_string(params)?;
        self.database_path = params.database.clone();

        let timeout = Duration::from_secs(params.connection_timeout.unwrap_or(5) as u64);
        let max_connections = params.max_connections.unwrap_or(5);

        let pool = SqlitePoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(timeout)
            .connect(&connection_string)
            .await
            .map_err(|e| {
                AppError::Database(crate::database::DatabaseError::ConnectionFailed(
                    e.to_string(),
                ))
            })?;

        // Enable foreign key constraints
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;

        self.pool = Some(pool);
        self.connected = true;

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), AppError> {
        if let Some(pool) = &self.pool {
            pool.close().await;
        }
        self.pool = None;
        self.connected = false;
        Ok(())
    }

    async fn test_connection(&self) -> Result<bool, AppError> {
        let pool = self.get_pool()?;

        match sqlx::query("SELECT 1")
            .fetch_one(pool)
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn execute_query(&self, query: &str) -> Result<QueryResult, AppError> {
        let pool = self.get_pool()?;

        let start = std::time::Instant::now();
        let rows: Vec<SqliteRow> = sqlx::query(query)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;

        let execution_time = start.elapsed().as_millis() as u64;

        // Get column information from the first row
        let columns = if let Some(first_row) = rows.first() {
            first_row
                .columns()
                .iter()
                .map(|col| ColumnInfo {
                    name: col.name().to_string(),
                    data_type: col.type_info().name().to_string(),
                    is_nullable: true, // SQLite doesn't track nullability well
                })
                .collect()
        } else {
            vec![]
        };

        // Convert rows to QueryRow
        let query_rows: Vec<QueryRow> = rows
            .iter()
            .map(|row| {
                let values: Vec<Option<String>> = (0..row.columns().len())
                    .map(|i| {
                        // Try to get value as string
                        // SQLite stores most things as TEXT, INTEGER, REAL, or BLOB
                        if let Ok(val) = row.try_get::<String, _>(i) {
                            Some(val)
                        } else if let Ok(val) = row.try_get::<i64, _>(i) {
                            Some(val.to_string())
                        } else if let Ok(val) = row.try_get::<f64, _>(i) {
                            Some(val.to_string())
                        } else if let Ok(val) = row.try_get::<bool, _>(i) {
                            Some(val.to_string())
                        } else {
                            None
                        }
                    })
                    .collect();

                QueryRow {
                    columns: row.columns().iter().map(|c| c.name().to_string()).collect(),
                    values,
                }
            })
            .collect();

        Ok(QueryResult {
            columns,
            rows: query_rows,
            rows_affected: None,
            execution_time: Some(execution_time),
        })
    }

    async fn execute_command(&self, command: &str) -> Result<u64, AppError> {
        let pool = self.get_pool()?;

        let result = sqlx::query(command)
            .execute(pool)
            .await
            .map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;

        Ok(result.rows_affected())
    }

    async fn begin_transaction(&mut self) -> Result<(), AppError> {
        // For now, we'll use implicit transactions with queries
        // Real transaction support would require storing transaction state
        Ok(())
    }

    async fn commit_transaction(&mut self) -> Result<(), AppError> {
        Ok(())
    }

    async fn rollback_transaction(&mut self) -> Result<(), AppError> {
        Ok(())
    }

    async fn get_metadata(&self) -> Result<DatabaseMetadata, AppError> {
        let pool = self.get_pool()?;

        // Get SQLite version
        let version_row = sqlx::query("SELECT sqlite_version()")
            .fetch_one(pool)
            .await
            .map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;

        let version: String = version_row.try_get(0).unwrap_or_else(|_| "Unknown".to_string());

        // Get database file size
        let size = if Path::new(&self.database_path).exists() {
            std::fs::metadata(&self.database_path)
                .ok()
                .map(|m| m.len() as i64)
        } else {
            None
        };

        // SQLite uses UTF-8 encoding by default
        let encoding = Some("UTF-8".to_string());

        Ok(DatabaseMetadata {
            version: format!("SQLite {}", version),
            database_name: Path::new(&self.database_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            size,
            encoding,
        })
    }

    async fn list_tables(&self) -> Result<Vec<TableInfo>, AppError> {
        let pool = self.get_pool()?;

        let rows = sqlx::query(
            r#"
            SELECT
                name,
                type
            FROM sqlite_master
            WHERE type IN ('table', 'view')
                AND name NOT LIKE 'sqlite_%'
            ORDER BY name
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
        })?;

        let mut tables = Vec::new();
        for row in rows {
            let name: String = row.try_get(0).map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;
            let table_type: String = row.try_get(1).map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;

            // Get row count for tables (not views)
            let row_count = if table_type == "table" {
                let count_query = format!("SELECT COUNT(*) FROM \"{}\"", name);
                let count_row = sqlx::query(&count_query)
                    .fetch_one(pool)
                    .await
                    .ok();

                count_row.and_then(|r| r.try_get::<i64, _>(0).ok())
            } else {
                None
            };

            tables.push(TableInfo {
                name,
                schema: None, // SQLite doesn't have schemas like PostgreSQL
                table_type: table_type.to_uppercase(),
                row_count,
            });
        }

        Ok(tables)
    }

    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<ColumnInfo>, AppError> {
        let pool = self.get_pool()?;

        // Use PRAGMA table_info to get column information
        let query = format!("PRAGMA table_info('{}')", table_name);

        let rows = sqlx::query(&query)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;

        let mut columns = Vec::new();
        for row in rows {
            let name: String = row.try_get(1).map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;
            let data_type: String = row.try_get(2).map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;
            let notnull: i64 = row.try_get(3).map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;

            columns.push(ColumnInfo {
                name,
                data_type,
                is_nullable: notnull == 0,
            });
        }

        Ok(columns)
    }

    async fn current_database(&self) -> Result<String, AppError> {
        Ok(Path::new(&self.database_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::SQLite
    }
    
    fn get_dialect(&self) -> Box<dyn SqlDialect> {
        Box::new(self.dialect.clone())
    }
    
    fn get_capabilities(&self) -> DatabaseCapabilities {
        DatabaseCapabilities::sqlite()
    }
    
    fn get_query_templates(&self) -> QueryTemplates {
        QueryTemplates::sqlite()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_string_building() {
        let params = ConnectionParams::new(DatabaseType::SQLite, "./database/sqlite/test.db".to_string());

        let conn_str = SqliteAdapter::build_connection_string(&params).unwrap();
        assert_eq!(conn_str, "sqlite://./database/sqlite/test.db?mode=rwc");
    }
}