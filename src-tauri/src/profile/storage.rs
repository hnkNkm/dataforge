use std::path::PathBuf;
use std::fs;
use keyring::Entry;
use serde_json;
use crate::error::AppError;
use super::{ConnectionProfile, crypto};

const APP_NAME: &str = "DataForge";
const PROFILE_FILE: &str = "profiles.encrypted";

/// Profile storage that handles saving/loading profiles and passwords
pub struct ProfileStorage {
    profiles_path: PathBuf,
}

impl ProfileStorage {
    /// Create a new profile storage instance
    pub fn new() -> Result<Self, AppError> {
        let profiles_path = Self::get_profiles_path()?;

        // Ensure the directory exists
        if let Some(parent) = profiles_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::Storage(format!("Failed to create profiles directory: {}", e))
            })?;
        }

        Ok(Self { profiles_path })
    }

    /// Get the path to the profiles file
    fn get_profiles_path() -> Result<PathBuf, AppError> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| AppError::Storage("Could not find home directory".to_string()))?;

        Ok(home_dir
            .join(".dataforge")
            .join("profiles")
            .join(PROFILE_FILE))
    }

    /// Save a profile to storage
    pub async fn save_profile(&self, profile: &ConnectionProfile) -> Result<(), AppError> {
        let mut profiles = self.load_all_profiles().await?;

        // Add or update the profile
        profiles.retain(|p| p.id != profile.id);
        profiles.push(profile.clone());

        self.save_all_profiles(&profiles).await
    }

    /// Update an existing profile
    pub async fn update_profile(&self, profile: &ConnectionProfile) -> Result<(), AppError> {
        let mut profiles = self.load_all_profiles().await?;

        // Find and update the profile
        let found = profiles.iter_mut().find(|p| p.id == profile.id);
        match found {
            Some(p) => *p = profile.clone(),
            None => return Err(AppError::Storage(format!("Profile {} not found", profile.id))),
        }

        self.save_all_profiles(&profiles).await
    }

    /// Delete a profile from storage
    pub async fn delete_profile(&self, id: &str) -> Result<(), AppError> {
        let mut profiles = self.load_all_profiles().await?;

        // Remove the profile
        profiles.retain(|p| p.id != id);

        self.save_all_profiles(&profiles).await
    }

    /// Get a specific profile by ID
    pub async fn get_profile(&self, id: &str) -> Result<ConnectionProfile, AppError> {
        let profiles = self.load_all_profiles().await?;

        profiles
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| AppError::Storage(format!("Profile {} not found", id)))
    }

    /// List all profiles
    pub async fn list_profiles(&self) -> Result<Vec<ConnectionProfile>, AppError> {
        self.load_all_profiles().await
    }

    /// Load all profiles from storage
    async fn load_all_profiles(&self) -> Result<Vec<ConnectionProfile>, AppError> {
        if !self.profiles_path.exists() {
            return Ok(Vec::new());
        }

        let encrypted_data = fs::read_to_string(&self.profiles_path).map_err(|e| {
            AppError::Storage(format!("Failed to read profiles file: {}", e))
        })?;

        if encrypted_data.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Decrypt the data
        let decrypted = crypto::decrypt(&encrypted_data)?;

        // Deserialize profiles
        let profiles: Vec<ConnectionProfile> = serde_json::from_slice(&decrypted).map_err(|e| {
            AppError::Storage(format!("Failed to deserialize profiles: {}", e))
        })?;

        Ok(profiles)
    }

    /// Save all profiles to storage
    async fn save_all_profiles(&self, profiles: &[ConnectionProfile]) -> Result<(), AppError> {
        // Serialize profiles
        let json_data = serde_json::to_vec(profiles).map_err(|e| {
            AppError::Storage(format!("Failed to serialize profiles: {}", e))
        })?;

        // Encrypt the data
        let encrypted = crypto::encrypt(&json_data)?;

        // Save to file
        fs::write(&self.profiles_path, encrypted).map_err(|e| {
            AppError::Storage(format!("Failed to write profiles file: {}", e))
        })?;

        Ok(())
    }

    /// Save a password to the OS keyring
    pub fn save_password(&self, profile_id: &str, password: &str) -> Result<(), AppError> {
        let entry = Entry::new(APP_NAME, &format!("profile_{}", profile_id))
            .map_err(|e| AppError::Storage(format!("Failed to access keyring: {}", e)))?;

        entry
            .set_password(password)
            .map_err(|e| AppError::Storage(format!("Failed to save password: {}", e)))?;

        Ok(())
    }

    /// Get a password from the OS keyring
    pub fn get_password(&self, profile_id: &str) -> Result<String, AppError> {
        let entry = Entry::new(APP_NAME, &format!("profile_{}", profile_id))
            .map_err(|e| AppError::Storage(format!("Failed to access keyring: {}", e)))?;

        entry
            .get_password()
            .map_err(|e| AppError::Storage(format!("Failed to get password: {}", e)))
    }

    /// Delete a password from the OS keyring
    pub fn delete_password(&self, profile_id: &str) -> Result<(), AppError> {
        let entry = Entry::new(APP_NAME, &format!("profile_{}", profile_id))
            .map_err(|e| AppError::Storage(format!("Failed to access keyring: {}", e)))?;

        // Try to delete, but don't fail if it doesn't exist
        let _ = entry.delete_credential();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_profile_storage() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let profiles_path = temp_dir.path().join("profiles.encrypted");

        // Create a test storage instance
        let storage = ProfileStorage {
            profiles_path: profiles_path.clone(),
        };

        // Create a test profile
        let profile = ConnectionProfile::new(
            "Test DB".to_string(),
            crate::database::adapter::DatabaseType::PostgreSQL,
            "testdb".to_string(),
        );

        // Save the profile
        storage.save_profile(&profile).await.unwrap();

        // Load profiles
        let profiles = storage.list_profiles().await.unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "Test DB");

        // Get specific profile
        let loaded_profile = storage.get_profile(&profile.id).await.unwrap();
        assert_eq!(loaded_profile.name, profile.name);

        // Delete profile
        storage.delete_profile(&profile.id).await.unwrap();
        let profiles = storage.list_profiles().await.unwrap();
        assert_eq!(profiles.len(), 0);
    }
}