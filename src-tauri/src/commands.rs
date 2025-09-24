use crate::database::{ConnectionParams, DatabaseAdapter, DatabaseType, create_adapter};
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// Global adapter storage (temporary - should use proper state management)
static mut ADAPTER: Option<Arc<Mutex<Box<dyn DatabaseAdapter>>>> = None;

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

    // Create adapter based on database type
    let mut adapter = create_adapter(params.database_type)
        .map_err(|e| format!("Failed to create adapter: {}", e))?;

    // Connect to database
    adapter.connect(&params).await
        .map_err(|e| format!("Connection failed: {}", e))?;

    // Store adapter globally (temporary solution)
    unsafe {
        ADAPTER = Some(Arc::new(Mutex::new(adapter)));
    }

    Ok("Connected successfully".to_string())
}

#[tauri::command]
pub async fn disconnect_database() -> Result<String, String> {
    unsafe {
        if let Some(adapter) = &ADAPTER {
            let mut adapter = adapter.lock().await;
            adapter.disconnect().await
                .map_err(|e| format!("Disconnect failed: {}", e))?;
        }
        ADAPTER = None;
    }

    Ok("Disconnected successfully".to_string())
}

#[tauri::command]
pub async fn test_database_connection_adapter() -> Result<bool, String> {
    unsafe {
        if let Some(adapter) = &ADAPTER {
            let adapter = adapter.lock().await;
            return adapter.test_connection().await
                .map_err(|e| format!("Test failed: {}", e));
        }
    }

    Err("No active connection".to_string())
}

#[tauri::command]
pub async fn execute_query(query: String) -> Result<serde_json::Value, String> {
    unsafe {
        if let Some(adapter) = &ADAPTER {
            let adapter = adapter.lock().await;
            let result = adapter.execute_query(&query).await
                .map_err(|e| format!("Query failed: {}", e))?;

            // Convert to JSON
            return serde_json::to_value(result)
                .map_err(|e| format!("Serialization failed: {}", e));
        }
    }

    Err("No active connection".to_string())
}

#[tauri::command]
pub async fn get_database_metadata() -> Result<serde_json::Value, String> {
    unsafe {
        if let Some(adapter) = &ADAPTER {
            let adapter = adapter.lock().await;
            let metadata = adapter.get_metadata().await
                .map_err(|e| format!("Failed to get metadata: {}", e))?;

            // Convert to JSON
            return serde_json::to_value(metadata)
                .map_err(|e| format!("Serialization failed: {}", e));
        }
    }

    Err("No active connection".to_string())
}

#[tauri::command]
pub async fn list_database_tables() -> Result<serde_json::Value, String> {
    unsafe {
        if let Some(adapter) = &ADAPTER {
            let adapter = adapter.lock().await;
            let tables = adapter.list_tables().await
                .map_err(|e| format!("Failed to list tables: {}", e))?;

            // Convert to JSON
            return serde_json::to_value(tables)
                .map_err(|e| format!("Serialization failed: {}", e));
        }
    }

    Err("No active connection".to_string())
}