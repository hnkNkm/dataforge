use async_trait::async_trait;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::{Column, Row, TypeInfo};
use std::time::Duration;

use super::{
    ColumnInfo, ConnectionParams, DatabaseAdapter, DatabaseMetadata, DatabaseType, QueryResult,
    QueryRow, TableInfo,
};
use crate::database::dialect::{SqlDialect, PostgreSQLDialect};
use crate::database::capabilities::{DatabaseCapabilities, QueryTemplates};
use crate::error::AppError;

pub struct PostgresAdapter {
    pool: Option<PgPool>,
    connected: bool,
    dialect: PostgreSQLDialect,
}

impl PostgresAdapter {
    pub fn new() -> Self {
        Self {
            pool: None,
            connected: false,
            dialect: PostgreSQLDialect::new(),
        }
    }

    fn get_pool(&self) -> Result<&PgPool, AppError> {
        self.pool
            .as_ref()
            .ok_or_else(|| AppError::Database(crate::database::DatabaseError::ConnectionFailed(
                "Not connected to database".to_string(),
            )))
    }

    fn build_connection_string(params: &ConnectionParams) -> String {
        let host = params.host.as_deref().unwrap_or("localhost");
        let port = params.port.unwrap_or(5432);
        let username = params.username.as_deref().unwrap_or("");
        let password = params.password.as_deref().unwrap_or("");
        let database = &params.database;

        let mut url = format!(
            "postgres://{}:{}@{}:{}/{}",
            username, password, host, port, database
        );

        // Add SSL mode if specified
        if let Some(ssl_mode) = &params.ssl_mode {
            url.push_str(&format!("?sslmode={}", ssl_mode));
        }

        url
    }
}

#[async_trait]
impl DatabaseAdapter for PostgresAdapter {
    async fn connect(&mut self, params: &ConnectionParams) -> Result<(), AppError> {
        params.validate()?;

        let connection_string = Self::build_connection_string(params);
        let timeout = Duration::from_secs(params.connection_timeout.unwrap_or(5) as u64);
        let max_connections = params.max_connections.unwrap_or(5);

        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(timeout)
            .connect(&connection_string)
            .await
            .map_err(|e| {
                AppError::Database(crate::database::DatabaseError::ConnectionFailed(
                    e.to_string(),
                ))
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
        let rows: Vec<PgRow> = sqlx::query(query)
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
                    is_nullable: true, // TODO: Get actual nullability
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
                        // Try different types to get the value as string
                        if let Ok(val) = row.try_get::<Option<String>, _>(i) {
                            val
                        } else if let Ok(val) = row.try_get::<Option<i32>, _>(i) {
                            val.map(|v| v.to_string())
                        } else if let Ok(val) = row.try_get::<Option<i64>, _>(i) {
                            val.map(|v| v.to_string())
                        } else if let Ok(val) = row.try_get::<Option<f64>, _>(i) {
                            val.map(|v| v.to_string())
                        } else if let Ok(val) = row.try_get::<Option<bool>, _>(i) {
                            val.map(|v| v.to_string())
                        } else if let Ok(val) = row.try_get::<Option<chrono::NaiveDateTime>, _>(i) {
                            val.map(|v| v.to_string())
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

        let version_row = sqlx::query("SELECT version()")
            .fetch_one(pool)
            .await
            .map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;

        let version: String = version_row.try_get(0).unwrap_or_else(|_| "Unknown".to_string());

        let db_name_row = sqlx::query("SELECT current_database()")
            .fetch_one(pool)
            .await
            .map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;

        let database_name: String = db_name_row.try_get(0).unwrap_or_else(|_| "Unknown".to_string());

        // Get database size
        let size_query = format!(
            "SELECT pg_database_size('{}') as size",
            database_name
        );
        let size_row = sqlx::query(&size_query)
            .fetch_one(pool)
            .await
            .ok();

        let size = size_row.and_then(|row| row.try_get::<i64, _>(0).ok());

        // Get encoding
        let encoding_row = sqlx::query("SELECT pg_encoding_to_char(encoding) FROM pg_database WHERE datname = current_database()")
            .fetch_one(pool)
            .await
            .ok();

        let encoding = encoding_row.and_then(|row| row.try_get::<String, _>(0).ok());

        Ok(DatabaseMetadata {
            version,
            database_name,
            size,
            encoding,
        })
    }

    async fn list_tables(&self) -> Result<Vec<TableInfo>, AppError> {
        let pool = self.get_pool()?;
        crate::log_info!("postgres_adapter", "Executing list_tables query");

        let rows = sqlx::query(
            r#"
            SELECT
                schemaname,
                tablename,
                CASE
                    WHEN schemaname = 'pg_catalog' OR schemaname = 'information_schema'
                    THEN 'SYSTEM'
                    ELSE 'TABLE'
                END as table_type
            FROM pg_tables
            WHERE schemaname NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
            ORDER BY schemaname, tablename
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            let error_msg = format!("Query failed: {}", e);
            crate::log_info!("postgres_adapter", "{}", error_msg);
            AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
        })?;

        crate::log_info!("postgres_adapter", "Found {} rows from pg_tables", rows.len());

        let mut tables = Vec::new();
        for row in rows {
            let schema: String = row.try_get(0).map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;
            let name: String = row.try_get(1).map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;
            let table_type: String = row.try_get(2).map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;

            crate::log_info!("postgres_adapter", "Found table: {}.{} (type: {})", schema, name, table_type);

            tables.push(TableInfo {
                name,
                schema: Some(schema),
                table_type,
                row_count: None, // Could be expensive to calculate
            });
        }

        Ok(tables)
    }

    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<ColumnInfo>, AppError> {
        let pool = self.get_pool()?;

        let query = r#"
            SELECT
                column_name,
                data_type,
                is_nullable
            FROM information_schema.columns
            WHERE table_name = $1
            ORDER BY ordinal_position
        "#;

        let rows = sqlx::query(query)
            .bind(table_name)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;

        let mut columns = Vec::new();
        for row in rows {
            let name: String = row.try_get(0).map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;
            let data_type: String = row.try_get(1).map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;
            let is_nullable: String = row.try_get(2).map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;

            columns.push(ColumnInfo {
                name,
                data_type,
                is_nullable: is_nullable == "YES",
            });
        }

        Ok(columns)
    }

    async fn current_database(&self) -> Result<String, AppError> {
        let pool = self.get_pool()?;

        let row = sqlx::query("SELECT current_database()")
            .fetch_one(pool)
            .await
            .map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;

        Ok(row.try_get(0).map_err(|e| {
            AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
        })?)
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn database_type(&self) -> DatabaseType {
        DatabaseType::PostgreSQL
    }
    
    fn get_dialect(&self) -> Box<dyn SqlDialect> {
        Box::new(self.dialect.clone())
    }
    
    fn get_capabilities(&self) -> DatabaseCapabilities {
        DatabaseCapabilities::postgresql()
    }
    
    fn get_query_templates(&self) -> QueryTemplates {
        QueryTemplates::postgresql()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_string_building() {
        let mut params = ConnectionParams::new(DatabaseType::PostgreSQL, "test_db".to_string());
        params.host = Some("localhost".to_string());
        params.port = Some(5432);
        params.username = Some("user".to_string());
        params.password = Some("pass".to_string());

        let conn_str = PostgresAdapter::build_connection_string(&params);
        assert_eq!(conn_str, "postgres://user:pass@localhost:5432/test_db");

        // Test with SSL
        params.ssl_mode = Some("require".to_string());
        let conn_str = PostgresAdapter::build_connection_string(&params);
        assert_eq!(conn_str, "postgres://user:pass@localhost:5432/test_db?sslmode=require");
    }
}