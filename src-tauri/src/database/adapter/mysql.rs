use async_trait::async_trait;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions, MySqlRow};
use sqlx::{Column, Row, TypeInfo};
use std::time::Duration;

use super::{
    ColumnInfo, ConnectionParams, DatabaseAdapter, DatabaseMetadata, DatabaseType, QueryResult,
    QueryRow, TableInfo,
};
use crate::database::dialect::{SqlDialect, MySQLDialect};
use crate::database::capabilities::{DatabaseCapabilities, QueryTemplates};
use crate::error::AppError;

pub struct MySqlAdapter {
    pool: Option<MySqlPool>,
    connected: bool,
    dialect: MySQLDialect,
}

impl MySqlAdapter {
    pub fn new() -> Self {
        Self {
            pool: None,
            connected: false,
            dialect: MySQLDialect::new(),
        }
    }

    fn get_pool(&self) -> Result<&MySqlPool, AppError> {
        self.pool
            .as_ref()
            .ok_or_else(|| AppError::Database(crate::database::DatabaseError::ConnectionFailed(
                "Not connected to database".to_string(),
            )))
    }

    fn build_connection_string(params: &ConnectionParams) -> String {
        let host = params.host.as_deref().unwrap_or("localhost");
        let port = params.port.unwrap_or(3306);
        let username = params.username.as_deref().unwrap_or("root");
        let password = params.password.as_deref().unwrap_or("");
        let database = &params.database;

        if password.is_empty() {
            format!(
                "mysql://{}@{}:{}/{}",
                username, host, port, database
            )
        } else {
            format!(
                "mysql://{}:{}@{}:{}/{}",
                username, password, host, port, database
            )
        }
    }
}

#[async_trait]
impl DatabaseAdapter for MySqlAdapter {
    async fn connect(&mut self, params: &ConnectionParams) -> Result<(), AppError> {
        params.validate()?;

        let connection_string = Self::build_connection_string(params);
        let timeout = Duration::from_secs(params.connection_timeout.unwrap_or(5) as u64);
        let max_connections = params.max_connections.unwrap_or(5);

        let pool = MySqlPoolOptions::new()
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
        let rows: Vec<MySqlRow> = sqlx::query(query)
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
                        // Try to get value as string
                        row.try_get::<Option<String>, _>(i)
                            .unwrap_or(None)
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

        let version_row = sqlx::query("SELECT VERSION()")
            .fetch_one(pool)
            .await
            .map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;

        let version: String = version_row.try_get(0).unwrap_or_else(|_| "Unknown".to_string());

        let db_name_row = sqlx::query("SELECT DATABASE()")
            .fetch_one(pool)
            .await
            .map_err(|e| {
                AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
            })?;

        let database_name: String = db_name_row.try_get(0).unwrap_or_else(|_| "Unknown".to_string());

        // Get database size
        let size_query = format!(
            r#"
            SELECT SUM(data_length + index_length) AS size
            FROM information_schema.tables
            WHERE table_schema = '{}'
            "#,
            database_name
        );
        let size_row = sqlx::query(&size_query)
            .fetch_one(pool)
            .await
            .ok();

        let size = size_row.and_then(|row| row.try_get::<Option<i64>, _>(0).ok().flatten());

        // Get encoding
        let encoding_row = sqlx::query(
            r#"
            SELECT default_character_set_name
            FROM information_schema.schemata
            WHERE schema_name = DATABASE()
            "#
        )
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

        let rows = sqlx::query(
            r#"
            SELECT
                CAST(TABLE_SCHEMA AS CHAR) AS TABLE_SCHEMA,
                CAST(TABLE_NAME AS CHAR) AS TABLE_NAME,
                CAST(TABLE_TYPE AS CHAR) AS TABLE_TYPE
            FROM information_schema.tables
            WHERE TABLE_SCHEMA = DATABASE()
            ORDER BY TABLE_NAME
            "#
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            AppError::Database(crate::database::DatabaseError::QueryFailed(e.to_string()))
        })?;

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

            // Get row count
            let count_query = format!("SELECT COUNT(*) FROM `{}`", name);
            let count_row = sqlx::query(&count_query)
                .fetch_one(pool)
                .await
                .ok();

            let row_count = count_row.and_then(|r| r.try_get::<i64, _>(0).ok());

            tables.push(TableInfo {
                name,
                schema: Some(schema),
                table_type,
                row_count,
            });
        }

        Ok(tables)
    }

    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<ColumnInfo>, AppError> {
        let pool = self.get_pool()?;

        let query = r#"
            SELECT
                COLUMN_NAME,
                DATA_TYPE,
                IS_NULLABLE
            FROM information_schema.columns
            WHERE TABLE_SCHEMA = DATABASE()
                AND TABLE_NAME = ?
            ORDER BY ORDINAL_POSITION
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

        let row = sqlx::query("SELECT DATABASE()")
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
        DatabaseType::MySQL
    }
    
    fn get_dialect(&self) -> Box<dyn SqlDialect> {
        Box::new(self.dialect.clone())
    }
    
    fn get_capabilities(&self) -> DatabaseCapabilities {
        DatabaseCapabilities::mysql()
    }
    
    fn get_query_templates(&self) -> QueryTemplates {
        QueryTemplates::mysql()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_string_building() {
        let mut params = ConnectionParams::new(DatabaseType::MySQL, "test_db".to_string());
        params.host = Some("localhost".to_string());
        params.port = Some(3306);
        params.username = Some("user".to_string());
        params.password = Some("pass".to_string());

        let conn_str = MySqlAdapter::build_connection_string(&params);
        assert_eq!(conn_str, "mysql://user:pass@localhost:3306/test_db");

        // Test without password
        params.password = None;
        let conn_str = MySqlAdapter::build_connection_string(&params);
        assert_eq!(conn_str, "mysql://user@localhost:3306/test_db");
    }
}