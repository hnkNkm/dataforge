use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: Option<u32>,
}

impl DatabaseConfig {
    pub fn new(
        host: String,
        port: u16,
        database: String,
        username: String,
        password: String,
    ) -> Self {
        Self {
            host,
            port,
            database,
            username,
            password,
            max_connections: Some(5),
        }
    }

    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            host: std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            database: std::env::var("DB_NAME")?,
            username: std::env::var("DB_USER")?,
            password: std::env::var("DB_PASSWORD")?,
            max_connections: Some(5),
        })
    }

    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}