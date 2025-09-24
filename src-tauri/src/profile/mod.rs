use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::database::adapter::{ConnectionParams, DatabaseType};
use crate::error::AppError;

pub mod storage;
pub mod crypto;

/// Connection profile that stores database connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub id: String,
    pub name: String,
    pub database_type: DatabaseType,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: String,
    pub username: Option<String>,
    pub ssl_mode: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_connected: Option<DateTime<Utc>>,
}

impl ConnectionProfile {
    /// Create a new connection profile
    pub fn new(name: String, database_type: DatabaseType, database: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            database_type,
            host: if database_type.requires_host() {
                Some("localhost".to_string())
            } else {
                None
            },
            port: database_type.default_port(),
            database,
            username: None,
            ssl_mode: None,
            color: None,
            icon: None,
            created_at: now,
            updated_at: now,
            last_connected: None,
        }
    }

    /// Convert profile to connection parameters
    pub fn to_connection_params(&self) -> ConnectionParams {
        ConnectionParams {
            database_type: self.database_type,
            host: self.host.clone(),
            port: self.port,
            database: self.database.clone(),
            username: self.username.clone(),
            password: None, // Password is retrieved separately from keyring
            ssl_mode: self.ssl_mode.clone(),
            connection_timeout: Some(5),
            max_connections: Some(5),
            additional_params: std::collections::HashMap::new(),
        }
    }

    /// Update the last connected timestamp
    pub fn update_last_connected(&mut self) {
        self.last_connected = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

/// Profile manager that handles all profile operations
pub struct ProfileManager {
    storage: storage::ProfileStorage,
}

impl ProfileManager {
    /// Create a new profile manager
    pub fn new() -> Result<Self, AppError> {
        let storage = storage::ProfileStorage::new()?;
        Ok(Self { storage })
    }

    /// Create and save a new profile
    pub async fn create_profile(&self, mut profile: ConnectionProfile, password: Option<String>) -> Result<ConnectionProfile, AppError> {
        // Save password to keyring if provided
        if let Some(pwd) = password {
            self.storage.save_password(&profile.id, &pwd)?;
        }

        // Save profile to storage
        self.storage.save_profile(&profile).await?;

        Ok(profile)
    }

    /// List all profiles
    pub async fn list_profiles(&self) -> Result<Vec<ConnectionProfile>, AppError> {
        self.storage.list_profiles().await
    }

    /// Get a specific profile by ID
    pub async fn get_profile(&self, id: &str) -> Result<ConnectionProfile, AppError> {
        self.storage.get_profile(id).await
    }

    /// Update an existing profile
    pub async fn update_profile(&self, mut profile: ConnectionProfile, password: Option<String>) -> Result<ConnectionProfile, AppError> {
        profile.updated_at = Utc::now();

        // Update password if provided
        if let Some(pwd) = password {
            self.storage.save_password(&profile.id, &pwd)?;
        }

        // Update profile in storage
        self.storage.update_profile(&profile).await?;

        Ok(profile)
    }

    /// Delete a profile
    pub async fn delete_profile(&self, id: &str) -> Result<(), AppError> {
        // Delete password from keyring
        self.storage.delete_password(id)?;

        // Delete profile from storage
        self.storage.delete_profile(id).await?;

        Ok(())
    }

    /// Get connection parameters with password for a profile
    pub async fn get_connection_params(&self, id: &str) -> Result<ConnectionParams, AppError> {
        let profile = self.get_profile(id).await?;
        let mut params = profile.to_connection_params();

        // Retrieve password from keyring
        if let Ok(password) = self.storage.get_password(id) {
            params.password = Some(password);
        }

        Ok(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_profile() {
        let profile = ConnectionProfile::new(
            "Test DB".to_string(),
            DatabaseType::PostgreSQL,
            "testdb".to_string(),
        );

        assert_eq!(profile.name, "Test DB");
        assert_eq!(profile.database_type, DatabaseType::PostgreSQL);
        assert_eq!(profile.database, "testdb");
        assert_eq!(profile.host, Some("localhost".to_string()));
        assert_eq!(profile.port, Some(5432));
        assert!(profile.id.len() > 0);
    }

    #[test]
    fn test_profile_to_connection_params() {
        let profile = ConnectionProfile::new(
            "Test DB".to_string(),
            DatabaseType::PostgreSQL,
            "testdb".to_string(),
        );

        let params = profile.to_connection_params();

        assert_eq!(params.database_type, DatabaseType::PostgreSQL);
        assert_eq!(params.database, "testdb");
        assert_eq!(params.host, Some("localhost".to_string()));
        assert_eq!(params.port, Some(5432));
        assert!(params.password.is_none());
    }
}