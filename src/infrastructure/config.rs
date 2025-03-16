use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;
use std::time::Duration;

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub environment: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let env = env::var("RUN_ENV").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            .set_default("environment", env.clone()).unwrap()
            // Start with default settings
            .add_source(File::with_name("config/default"))
            // Add environment-specific settings
            .add_source(File::with_name(&format!("config/{}", env)).required(false))
            // Add local settings (not in version control)
            .add_source(File::with_name("config/local").required(false))
            // Override with environment variables (APP_DATABASE_URL, etc.)
            .add_source(Environment::with_prefix("APP").separator("_"))
            .build()?;

        // Deserialize
        let settings: Settings = s.try_deserialize()?;

        Ok(settings)
    }

    pub fn database_connection_timeout(&self) -> Duration {
        Duration::from_secs(self.database.connection_timeout)
    }

}
