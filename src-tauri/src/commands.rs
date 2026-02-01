use crate::database::adapter::{ConnectionParams, DatabaseAdapter, DatabaseType, create_adapter};
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use once_cell::sync::Lazy;

pub mod profile;

// Global adapter storage using Lazy static
pub static ADAPTER_STATE: Lazy<Arc<Mutex<Option<Box<dyn DatabaseAdapter + Send + Sync>>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(None))
});

// Global connection cancellation token
pub static CONNECTION_CANCEL_TOKEN: Lazy<Arc<Mutex<Option<CancellationToken>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(None))
});

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub database_type: DatabaseType,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ssl_mode: Option<String>,
}

impl From<ConnectRequest> for ConnectionParams {
    fn from(req: ConnectRequest) -> Self {
        let mut params = ConnectionParams::new(req.database_type, req.database);
        params.host = req.host;
        params.port = req.port;
        params.username = req.username;
        params.password = req.password;
        params.ssl_mode = req.ssl_mode;
        params
    }
}

#[tauri::command]
pub async fn connect_database(request: ConnectRequest) -> Result<String, String> {
    let params: ConnectionParams = request.into();

    // Validate parameters
    if let Err(e) = params.validate() {
        return Err(format!("Validation error: {}", e));
    }

    // Create a new cancellation token
    let cancel_token = CancellationToken::new();
    let cancel_token_clone = cancel_token.clone();

    // Store the cancellation token
    {
        let mut token_state = CONNECTION_CANCEL_TOKEN.lock().await;
        *token_state = Some(cancel_token_clone);
    }

    // Create adapter based on database type
    let mut adapter = create_adapter(params.database_type)
        .map_err(|e| format!("Failed to create adapter: {}", e))?;

    // Connect to database with cancellation support
    let connect_result = tokio::select! {
        result = adapter.connect(&params) => result,
        _ = cancel_token.cancelled() => {
            // Clear the cancellation token
            let mut token_state = CONNECTION_CANCEL_TOKEN.lock().await;
            *token_state = None;
            return Err("Connection cancelled by user".to_string());
        }
    };

    // Clear the cancellation token
    {
        let mut token_state = CONNECTION_CANCEL_TOKEN.lock().await;
        *token_state = None;
    }

    connect_result.map_err(|e| format!("Connection failed: {}", e))?;

    // Store adapter in global state
    let mut adapter_state = ADAPTER_STATE.lock().await;
    *adapter_state = Some(adapter);

    Ok("Connected successfully".to_string())
}

#[tauri::command]
pub async fn disconnect_database() -> Result<String, String> {
    // Take the adapter out of the mutex
    let adapter_option = {
        let mut adapter_state = ADAPTER_STATE.lock().await;
        adapter_state.take()
    };

    if let Some(mut adapter) = adapter_option {
        adapter.disconnect().await
            .map_err(|e| format!("Disconnect failed: {}", e))?;
    }

    Ok("Disconnected successfully".to_string())
}

#[tauri::command]
pub async fn test_database_connection_adapter() -> Result<bool, String> {
    let adapter_state = ADAPTER_STATE.lock().await;

    if let Some(adapter) = adapter_state.as_ref() {
        return adapter.test_connection().await
            .map_err(|e| format!("Test failed: {}", e));
    }

    Err("No active connection".to_string())
}

#[tauri::command]
pub async fn execute_query(query: String) -> Result<serde_json::Value, String> {
    let adapter_state = ADAPTER_STATE.lock().await;

    if let Some(adapter) = adapter_state.as_ref() {
        // Get database type for SQL parsing
        let db_type = adapter.database_type();

        // Split SQL statements
        let statements = crate::database::sql_utils::split_sql_statements(&query, &db_type)
            .map_err(|e| format!("Failed to parse SQL: {}", e))?;

        if statements.is_empty() {
            return Err("No valid SQL statements found".to_string());
        }

        let mut results = Vec::new();
        let mut total_execution_time = 0u64;
        let mut total_rows_affected = 0u64;

        // Execute each statement
        for statement in statements {
            let trimmed = statement.trim();
            if trimmed.is_empty() {
                continue;
            }

            let start = std::time::Instant::now();

            // Try to execute as query first (SELECT, SHOW, etc.)
            match adapter.execute_query(trimmed).await {
                Ok(result) => {
                    let exec_time = start.elapsed().as_millis() as u64;
                    total_execution_time += exec_time;

                    // Transform rows from array format to object format
                    let transformed_rows: Vec<serde_json::Value> = result.rows.iter().map(|row| {
                        let mut obj = serde_json::Map::new();
                        for (i, column) in row.columns.iter().enumerate() {
                            let value = row.values.get(i)
                                .and_then(|v| v.as_ref())
                                .map(|v| serde_json::Value::String(v.clone()))
                                .unwrap_or(serde_json::Value::Null);
                            obj.insert(column.clone(), value);
                        }
                        serde_json::Value::Object(obj)
                    }).collect();

                    results.push(serde_json::json!({
                        "type": "query",
                        "statement": trimmed,
                        "columns": result.columns,
                        "rows": transformed_rows,
                        "rows_affected": result.rows_affected,
                        "execution_time": exec_time
                    }));
                }
                Err(_) => {
                    // If query fails, try as command (INSERT, UPDATE, DELETE, etc.)
                    match adapter.execute_command(trimmed).await {
                        Ok(affected) => {
                            let exec_time = start.elapsed().as_millis() as u64;
                            total_execution_time += exec_time;
                            total_rows_affected += affected;

                            results.push(serde_json::json!({
                                "type": "command",
                                "statement": trimmed,
                                "rows_affected": affected,
                                "execution_time": exec_time
                            }));
                        }
                        Err(e) => {
                            return Err(format!("Failed to execute statement: {}\nStatement: {}", e, trimmed));
                        }
                    }
                }
            }
        }

        // Return results
        if results.is_empty() {
            return Err("No results from execution".to_string());
        }

        // If single result and it's a query, return in backward-compatible format
        if results.len() == 1 {
            if let Some(first) = results.first() {
                if first["type"] == "query" {
                    return Ok(serde_json::json!({
                        "columns": first["columns"],
                        "rows": first["rows"],
                        "rows_affected": first["rows_affected"],
                        "execution_time": first["execution_time"]
                    }));
                }
            }
        }

        // Return multiple results
        return Ok(serde_json::json!({
            "results": results,
            "total_execution_time": total_execution_time,
            "total_rows_affected": total_rows_affected
        }));
    }

    Err("No active connection".to_string())
}

#[tauri::command]
pub async fn get_database_metadata() -> Result<serde_json::Value, String> {
    let adapter_state = ADAPTER_STATE.lock().await;

    if let Some(adapter) = adapter_state.as_ref() {
        let metadata = adapter.get_metadata().await
            .map_err(|e| format!("Failed to get metadata: {}", e))?;

        // Convert to JSON
        return serde_json::to_value(metadata)
            .map_err(|e| format!("Serialization failed: {}", e));
    }

    Err("No active connection".to_string())
}

#[tauri::command]
pub async fn list_database_tables() -> Result<serde_json::Value, String> {
    let adapter_state = ADAPTER_STATE.lock().await;

    if let Some(adapter) = adapter_state.as_ref() {
        crate::log_info!("command", "Fetching database tables...");
        let tables = adapter.list_tables().await
            .map_err(|e| {
                let error_msg = format!("Failed to list tables: {}", e);
                crate::log_info!("command", "{}", error_msg);
                error_msg
            })?;

        crate::log_info!("command", "Found {} tables", tables.len());

        // Convert to JSON
        let json_value = serde_json::to_value(tables)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        crate::log_info!("command", "Returning tables JSON: {:?}", json_value);
        return Ok(json_value);
    }

    crate::log_info!("command", "No active connection");
    Err("No active connection".to_string())
}

#[tauri::command]
pub async fn cancel_connection() -> Result<String, String> {
    let mut token_state = CONNECTION_CANCEL_TOKEN.lock().await;

    if let Some(token) = token_state.take() {
        token.cancel();
        Ok("Connection cancellation requested".to_string())
    } else {
        Err("No active connection to cancel".to_string())
    }
}

#[tauri::command]
pub async fn get_table_indexes(table_name: String) -> Result<serde_json::Value, String> {
    let adapter_state = ADAPTER_STATE.lock().await;

    if let Some(adapter) = adapter_state.as_ref() {
        crate::log_info!("command", "Fetching indexes for table: {}", table_name);
        
        // Get indexes using raw SQL query based on database type
        let query = match adapter.database_type() {
            DatabaseType::PostgreSQL => {
                format!(
                    "SELECT 
                        i.indexname AS index_name,
                        i.indexdef AS definition,
                        CASE 
                            WHEN i.indexname LIKE '%_pkey' THEN true 
                            ELSE false 
                        END AS is_primary,
                        CASE 
                            WHEN i.indexdef LIKE '%UNIQUE%' THEN true 
                            ELSE false 
                        END AS is_unique,
                        pg_size_pretty(pg_relation_size(c.oid)) AS size
                    FROM pg_indexes i
                    LEFT JOIN pg_class c ON c.relname = i.indexname
                    WHERE i.tablename = '{}'
                    ORDER BY i.indexname",
                    table_name
                )
            },
            DatabaseType::MySQL => {
                format!(
                    "SELECT 
                        INDEX_NAME AS index_name,
                        COLUMN_NAME AS column_name,
                        CASE 
                            WHEN INDEX_NAME = 'PRIMARY' THEN true 
                            ELSE false 
                        END AS is_primary,
                        CASE 
                            WHEN NON_UNIQUE = 0 THEN true 
                            ELSE false 
                        END AS is_unique,
                        INDEX_TYPE AS index_type,
                        CARDINALITY AS cardinality
                    FROM information_schema.STATISTICS
                    WHERE TABLE_NAME = '{}'
                    ORDER BY INDEX_NAME, SEQ_IN_INDEX",
                    table_name
                )
            },
            DatabaseType::SQLite => {
                format!(
                    "SELECT 
                        name AS index_name,
                        sql AS definition,
                        CASE 
                            WHEN sql LIKE '%PRIMARY KEY%' THEN true 
                            ELSE false 
                        END AS is_primary,
                        CASE 
                            WHEN sql LIKE '%UNIQUE%' THEN true 
                            ELSE false 
                        END AS is_unique
                    FROM sqlite_master
                    WHERE type = 'index' 
                    AND tbl_name = '{}'
                    ORDER BY name",
                    table_name
                )
            },
        };
        
        let result = adapter.execute_query(&query).await
            .map_err(|e| format!("Failed to get indexes: {}", e))?;
        
        // Convert QueryResult to JSON format compatible with frontend
        let json_result = serde_json::json!({
            "columns": result.columns,
            "rows": result.rows.iter().map(|row| {
                let mut obj = serde_json::Map::new();
                for (i, col) in result.columns.iter().enumerate() {
                    if let Some(value) = row.values.get(i) {
                        obj.insert(col.name.clone(), 
                            value.as_ref().map_or(serde_json::Value::Null, |v| serde_json::Value::String(v.clone())));
                    }
                }
                serde_json::Value::Object(obj)
            }).collect::<Vec<_>>(),
            "rows_affected": result.rows_affected,
            "execution_time": result.execution_time
        });
        
        crate::log_info!("command", "Found indexes for table {}", table_name);
        return Ok(json_result);
    }

    Err("No active connection".to_string())
}

#[tauri::command]
pub async fn generate_select_query(table_name: String) -> Result<String, String> {
    let adapter_state = ADAPTER_STATE.lock().await;

    if let Some(adapter) = adapter_state.as_ref() {
        // Get table columns
        let columns_query = match adapter.database_type() {
            DatabaseType::PostgreSQL => {
                format!(
                    "SELECT column_name 
                    FROM information_schema.columns 
                    WHERE table_name = '{}' 
                    ORDER BY ordinal_position",
                    table_name
                )
            },
            DatabaseType::MySQL => {
                format!(
                    "SELECT COLUMN_NAME AS column_name 
                    FROM information_schema.COLUMNS 
                    WHERE TABLE_NAME = '{}' 
                    ORDER BY ORDINAL_POSITION",
                    table_name
                )
            },
            DatabaseType::SQLite => {
                format!("PRAGMA table_info({})", table_name)
            },
        };
        
        let result = adapter.execute_query(&columns_query).await
            .map_err(|e| format!("Failed to get columns: {}", e))?;
        
        // Extract column names from QueryResult
        let columns: Vec<String> = if adapter.database_type() == DatabaseType::SQLite {
            // SQLite PRAGMA returns different structure
            result.rows.iter()
                .filter_map(|row| {
                    // Find the index of 'name' column
                    result.columns.iter().position(|col| col.name == "name")
                        .and_then(|idx| row.values.get(idx))
                        .and_then(|v| v.as_ref())
                        .map(|s| s.to_string())
                })
                .collect()
        } else {
            // PostgreSQL and MySQL
            result.rows.iter()
                .filter_map(|row| {
                    // Find the index of 'column_name' column
                    result.columns.iter().position(|col| col.name == "column_name")
                        .and_then(|idx| row.values.get(idx))
                        .and_then(|v| v.as_ref())
                        .map(|s| s.to_string())
                })
                .collect()
        };
        
        if columns.is_empty() {
            return Ok(format!("SELECT * FROM {} LIMIT 100;", table_name));
        }
        
        // Generate formatted SELECT query
        let select_query = format!(
            "SELECT\n    {}\nFROM {}\nLIMIT 100;",
            columns.join(",\n    "),
            table_name
        );
        
        return Ok(select_query);
    }

    Err("No active connection".to_string())
}