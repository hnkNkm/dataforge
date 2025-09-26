use serde::Deserialize;
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::profile::{ConnectionProfile, ProfileManager};
use crate::database::adapter::DatabaseType;

/// Request structure for creating a profile
#[derive(Debug, Deserialize)]
pub struct CreateProfileRequest {
    pub name: String,
    pub database_type: DatabaseType,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ssl_mode: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

/// Request structure for updating a profile
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub id: String,
    pub name: String,
    pub database_type: DatabaseType,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ssl_mode: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

/// Profile manager state for Tauri
pub struct ProfileManagerState(pub Arc<Mutex<Option<ProfileManager>>>);

impl ProfileManagerState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }
}

/// Create a new connection profile
#[tauri::command]
pub async fn create_profile(
    request: CreateProfileRequest,
    state: State<'_, ProfileManagerState>,
) -> Result<ConnectionProfile, String> {
    let mut manager_guard = state.0.lock().await;

    if manager_guard.is_none() {
        *manager_guard = Some(ProfileManager::new().map_err(|e| e.to_string())?);
    }

    let manager = manager_guard.as_ref().ok_or("Profile manager not initialized")?;

    let mut profile = ConnectionProfile::new(
        request.name,
        request.database_type,
        request.database,
    );

    // Set optional fields
    if let Some(host) = request.host {
        profile.host = Some(host);
    }
    if let Some(port) = request.port {
        profile.port = Some(port);
    }
    if let Some(username) = request.username {
        profile.username = Some(username);
    }
    if let Some(ssl_mode) = request.ssl_mode {
        profile.ssl_mode = Some(ssl_mode);
    }
    if let Some(color) = request.color {
        profile.color = Some(color);
    }
    if let Some(icon) = request.icon {
        profile.icon = Some(icon);
    }

    manager.create_profile(profile, request.password)
        .await
        .map_err(|e| e.to_string())
}

/// List all connection profiles
#[tauri::command]
pub async fn list_profiles(
    state: State<'_, ProfileManagerState>,
) -> Result<Vec<ConnectionProfile>, String> {
    let mut manager_guard = state.0.lock().await;

    if manager_guard.is_none() {
        *manager_guard = Some(ProfileManager::new().map_err(|e| e.to_string())?);
    }

    let manager = manager_guard.as_ref().ok_or("Profile manager not initialized")?;

    manager.list_profiles()
        .await
        .map_err(|e| e.to_string())
}

/// Get a specific profile by ID
#[tauri::command]
pub async fn get_profile(
    id: String,
    state: State<'_, ProfileManagerState>,
) -> Result<ConnectionProfile, String> {
    let mut manager_guard = state.0.lock().await;

    if manager_guard.is_none() {
        *manager_guard = Some(ProfileManager::new().map_err(|e| e.to_string())?);
    }

    let manager = manager_guard.as_ref().ok_or("Profile manager not initialized")?;

    manager.get_profile(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Update an existing profile
#[tauri::command]
pub async fn update_profile(
    request: UpdateProfileRequest,
    state: State<'_, ProfileManagerState>,
) -> Result<ConnectionProfile, String> {
    let mut manager_guard = state.0.lock().await;

    if manager_guard.is_none() {
        *manager_guard = Some(ProfileManager::new().map_err(|e| e.to_string())?);
    }

    let manager = manager_guard.as_ref().ok_or("Profile manager not initialized")?;

    // Get the existing profile to preserve created_at and other metadata
    let mut profile = manager.get_profile(&request.id)
        .await
        .map_err(|e| e.to_string())?;

    // Update fields
    profile.name = request.name;
    profile.database_type = request.database_type;
    profile.database = request.database;
    profile.host = request.host;
    profile.port = request.port;
    profile.username = request.username;
    profile.ssl_mode = request.ssl_mode;
    profile.color = request.color;
    profile.icon = request.icon;

    manager.update_profile(profile, request.password)
        .await
        .map_err(|e| e.to_string())
}

/// Delete a profile
#[tauri::command]
pub async fn delete_profile(
    id: String,
    state: State<'_, ProfileManagerState>,
) -> Result<(), String> {
    let mut manager_guard = state.0.lock().await;

    if manager_guard.is_none() {
        *manager_guard = Some(ProfileManager::new().map_err(|e| e.to_string())?);
    }

    let manager = manager_guard.as_ref().ok_or("Profile manager not initialized")?;

    manager.delete_profile(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Connect to a database using a profile
#[tauri::command]
pub async fn connect_with_profile(
    profile_id: String,
    state: State<'_, ProfileManagerState>,
) -> Result<String, String> {
    use crate::database::adapter::create_adapter;
    use crate::commands::{ADAPTER_STATE, CONNECTION_CANCEL_TOKEN};
    use tokio_util::sync::CancellationToken;

    let mut manager_guard = state.0.lock().await;

    if manager_guard.is_none() {
        *manager_guard = Some(ProfileManager::new().map_err(|e| e.to_string())?);
    }

    let manager = manager_guard.as_ref().ok_or("Profile manager not initialized")?;

    // Get connection parameters with password
    let params = manager.get_connection_params(&profile_id)
        .await
        .map_err(|e| e.to_string())?;

    // Get the profile to update last_connected
    let mut profile = manager.get_profile(&profile_id)
        .await
        .map_err(|e| e.to_string())?;

    // Create a new cancellation token
    let cancel_token = CancellationToken::new();
    let cancel_token_clone = cancel_token.clone();

    // Store the cancellation token
    {
        let mut token_state = CONNECTION_CANCEL_TOKEN.lock().await;
        *token_state = Some(cancel_token_clone);
    }

    // Create adapter and connect with cancellation support
    let mut adapter = create_adapter(params.database_type).map_err(|e| e.to_string())?;

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

    connect_result.map_err(|e| e.to_string())?;

    // Store the adapter in global state
    let mut adapter_state = ADAPTER_STATE.lock().await;
    *adapter_state = Some(adapter);

    // Update last connected timestamp
    profile.update_last_connected();
    manager.update_profile(profile.clone(), None)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "Connected to {} ({})",
        profile.name,
        profile.database
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_profile_request() {
        let request = CreateProfileRequest {
            name: "Test DB".to_string(),
            database_type: DatabaseType::PostgreSQL,
            host: Some("localhost".to_string()),
            port: Some(5432),
            database: "testdb".to_string(),
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            ssl_mode: None,
            color: None,
            icon: None,
        };

        assert_eq!(request.name, "Test DB");
        assert_eq!(request.database_type, DatabaseType::PostgreSQL);
        assert_eq!(request.port, Some(5432));
    }
}