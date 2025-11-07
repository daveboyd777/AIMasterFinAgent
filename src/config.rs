use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database connection configuration
    pub database: DatabaseConfig,

    /// Quicken integration settings
    pub quicken: QuickenConfig,

    /// AI analysis settings
    pub ai: AiConfig,

    /// Logging configuration
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Path to the SQLite database file
    pub path: PathBuf,

    /// Maximum number of database connections
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickenConfig {
    /// Path to monitor for Quicken QIF files
    pub watch_directory: PathBuf,

    /// File patterns to match for QIF imports
    pub file_patterns: Vec<String>,

    /// Auto-import new files
    pub auto_import: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// Enable AI-powered analysis
    pub enabled: bool,

    /// API endpoint for AI services
    pub api_endpoint: Option<String>,

    /// API key for AI services
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Log to file
    pub file_logging: bool,

    /// Log file path
    pub log_file: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        let project_dirs = ProjectDirs::from("com", "qspec", "fin-agent")
            .expect("Failed to get project directories");

        let data_dir = project_dirs.data_dir().to_path_buf();
        let _config_dir = project_dirs.config_dir().to_path_buf();

        Self {
            database: DatabaseConfig {
                path: data_dir.join("qspec_fin_agent.db"),
                max_connections: 5,
            },
            quicken: QuickenConfig {
                watch_directory: dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("Documents")
                    .join("Quicken"),
                file_patterns: vec!["*.qif".to_string(), "*.QIF".to_string()],
                auto_import: true,
            },
            ai: AiConfig {
                enabled: false,
                api_endpoint: None,
                api_key: None,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_logging: true,
                log_file: Some(data_dir.join("qspec_fin_agent.log")),
            },
        }
    }
}

impl Config {
    /// Load configuration from file or create default
    pub async fn load() -> Result<Self> {
        let project_dirs = ProjectDirs::from("com", "qspec", "fin-agent")
            .context("Failed to get project directories")?;

        let config_path = project_dirs.config_dir().join("config.toml");

        if config_path.exists() {
            let config_str = tokio::fs::read_to_string(&config_path)
                .await
                .context("Failed to read config file")?;

            let config: Config =
                toml::from_str(&config_str).context("Failed to parse config file")?;

            Ok(config)
        } else {
            let config = Config::default();
            config.save().await?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub async fn save(&self) -> Result<()> {
        let project_dirs = ProjectDirs::from("com", "qspec", "fin-agent")
            .context("Failed to get project directories")?;

        let config_dir = project_dirs.config_dir();
        tokio::fs::create_dir_all(config_dir)
            .await
            .context("Failed to create config directory")?;

        let config_path = config_dir.join("config.toml");
        let config_str = toml::to_string_pretty(self).context("Failed to serialize config")?;

        tokio::fs::write(&config_path, config_str)
            .await
            .context("Failed to write config file")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(!config.ai.enabled);
        assert_eq!(config.logging.level, "info");
        assert_eq!(config.database.max_connections, 5);
    }

    #[tokio::test]
    async fn test_config_load_and_save() {
        let _temp_dir = tempdir().expect("Failed to create temp dir");

        // Create a default config
        let _config = Config::default();

        // Test that we can create a config (load will create default if none exists)
        let loaded_config = Config::load().await;
        assert!(loaded_config.is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let serialized = toml::to_string(&config);
        assert!(serialized.is_ok());

        let deserialized: Result<Config, _> = toml::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
    }
}
