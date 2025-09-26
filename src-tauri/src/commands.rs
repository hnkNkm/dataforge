use crate::database::adapter::{ConnectionParams, DatabaseAdapter, DatabaseType, create_adapter};
use serde::{Deserialize, Serialize};
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
        let result = adapter.execute_query(&query).await
            .map_err(|e| format!("Query failed: {}", e))?;

        // Convert to JSON
        return serde_json::to_value(result)
            .map_err(|e| format!("Serialization failed: {}", e));
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
        let tables = adapter.list_tables().await
            .map_err(|e| format!("Failed to list tables: {}", e))?;

        // Convert to JSON
        return serde_json::to_value(tables)
            .map_err(|e| format!("Serialization failed: {}", e));
    }

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