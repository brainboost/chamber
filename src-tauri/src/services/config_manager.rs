use crate::models::config::ChamberConfig;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(workspace_path: &str) -> Self {
        let config_path = PathBuf::from(workspace_path)
            .join("config")
            .join("chamber-config.yaml");
        Self { config_path }
    }

    pub fn load_config(&self) -> Result<ChamberConfig> {
        if !self.config_path.exists() {
            // Create default config if it doesn't exist
            let default_config = ChamberConfig::default();
            self.save_config(&default_config)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&self.config_path)
            .context("Failed to read config file")?;

        let mut config: ChamberConfig = serde_yaml::from_str(&content)
            .context("Failed to parse config YAML")?;

        config.expand_env_vars();

        Ok(config)
    }

    pub fn save_config(&self, config: &ChamberConfig) -> Result<()> {
        // Ensure config directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let yaml = serde_yaml::to_string(config)
            .context("Failed to serialize config to YAML")?;

        fs::write(&self.config_path, yaml)
            .context("Failed to write config file")?;

        Ok(())
    }

    pub fn validate_config(config: &ChamberConfig) -> Result<()> {
        // Validate orchestrator
        if config.orchestrator.provider.is_empty() {
            anyhow::bail!("Orchestrator provider cannot be empty");
        }
        if config.orchestrator.model.is_empty() {
            anyhow::bail!("Orchestrator model cannot be empty");
        }

        // Validate at least one reasoning model is enabled
        let enabled_count = config.reasoning_models.iter()
            .filter(|m| m.enabled)
            .count();

        if enabled_count == 0 {
            anyhow::bail!("At least one reasoning model must be enabled");
        }

        // Validate workspace path
        if config.workspace.path.is_empty() {
            anyhow::bail!("Workspace path cannot be empty");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_default_config() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp_dir.path().to_str().unwrap());

        let config = manager.load_config().unwrap();
        assert_eq!(config.orchestrator.provider, "anthropic");
        assert!(!config.reasoning_models.is_empty());
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp_dir.path().to_str().unwrap());

        let mut config = ChamberConfig::default();
        config.orchestrator.model = "custom-model".to_string();

        manager.save_config(&config).unwrap();
        let loaded = manager.load_config().unwrap();

        assert_eq!(loaded.orchestrator.model, "custom-model");
    }

    #[test]
    fn test_validate_config() {
        let config = ChamberConfig::default();
        assert!(ConfigManager::validate_config(&config).is_ok());

        let mut invalid_config = ChamberConfig::default();
        invalid_config.orchestrator.provider = "".to_string();
        assert!(ConfigManager::validate_config(&invalid_config).is_err());
    }
}
