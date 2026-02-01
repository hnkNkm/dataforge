use tauri::State;
use serde_json::json;
use crate::commands::ADAPTER_STATE;
use crate::database::capabilities::{DatabaseCapabilities, QueryTemplates};

/// Get database capabilities for the current connection
#[tauri::command]
pub async fn get_database_capabilities() -> Result<DatabaseCapabilities, String> {
    let adapter_guard = ADAPTER_STATE.lock().await;
    
    if let Some(adapter) = adapter_guard.as_ref() {
        Ok(adapter.get_capabilities())
    } else {
        Err("Not connected to any database".to_string())
    }
}

/// Get query templates for the current database type
#[tauri::command]
pub async fn get_query_templates() -> Result<QueryTemplates, String> {
    let adapter_guard = ADAPTER_STATE.lock().await;
    
    if let Some(adapter) = adapter_guard.as_ref() {
        Ok(adapter.get_query_templates())
    } else {
        Err("Not connected to any database".to_string())
    }
}

/// Get database dialect information
#[tauri::command]
pub async fn get_dialect_info() -> Result<serde_json::Value, String> {
    let adapter_guard = ADAPTER_STATE.lock().await;
    
    if let Some(adapter) = adapter_guard.as_ref() {
        let dialect = adapter.get_dialect();
        
        Ok(json!({
            "quote_char": dialect.quote_identifier("test").chars().nth(0),
            "supports_schemas": dialect.supports_schemas(),
            "supports_returning": dialect.supports_returning_clause(),
            "boolean_true": dialect.boolean_literal(true),
            "boolean_false": dialect.boolean_literal(false),
            "current_timestamp": dialect.current_timestamp(),
            "auto_increment": dialect.auto_increment_type(),
        }))
    } else {
        Err("Not connected to any database".to_string())
    }
}