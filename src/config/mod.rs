use std::path::Path;

use anyhow::{Context, Result};

use tracing::{debug, info};

pub mod model;

pub use model::*;

impl Config {
    /// Load configuration from a specific file path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        info!("Loading configuration from: {}", path.display());

        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        config.validate()?;
        debug!("Configuration loaded and validated successfully");

        Ok(config)
    }

    /// Load configuration from default locations
    pub fn load_default() -> Result<Self> {
        use directories::ProjectDirs;

        if let Some(proj_dirs) = ProjectDirs::from("com", "contui", "contui") {
            let config_dir = proj_dirs.config_dir();
            let config_path = config_dir.join("config.toml");

            if config_path.exists() {
                return Self::load(&config_path);
            }
        }

        // Try current directory
        let local_config = std::path::PathBuf::from("config.toml");
        if local_config.exists() {
            return Self::load(&local_config);
        }

        info!("No configuration file found, using defaults");
        Ok(Config::default())
    }

    /// Save configuration to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        info!("Saving configuration to: {}", path.display());

        let contents = toml::to_string_pretty(self).context("Failed to serialize configuration")?;

        std::fs::write(path, contents)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        info!("Configuration saved successfully");
        Ok(())
    }

    /// Validate the configuration
    fn validate(&self) -> Result<()> {
        if self.general.poll_interval_ms < 100 {
            anyhow::bail!("poll_interval_ms must be at least 100");
        }

        if self.general.metrics_retention_seconds < 60 {
            anyhow::bail!("metrics_retention_seconds must be at least 60");
        }

        if let Some(threshold) = self.monitoring.cpu_threshold {
            if !(0.0..=100.0).contains(&threshold) {
                anyhow::bail!("cpu_threshold must be between 0 and 100");
            }
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            ui: UiConfig::default(),
            docker: DockerConfig::default(),
            keybindings: KeyBindings::default(),
            registries: vec![],
            monitoring: MonitoringConfig::default(),
            logging: LogConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.poll_interval_ms, 1000);
        assert_eq!(config.general.metrics_retention_seconds, 3600);
    }

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        assert!(config.validate().is_ok());

        let invalid_config = Config {
            general: GeneralConfig {
                poll_interval_ms: 50, // Too low
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_config_save_and_load() {
        let config = Config::default();
        let temp_file = NamedTempFile::new().unwrap();

        config.save(temp_file.path()).unwrap();

        let loaded = Config::load(temp_file.path()).unwrap();
        assert_eq!(
            loaded.general.poll_interval_ms,
            config.general.poll_interval_ms
        );
    }
}
