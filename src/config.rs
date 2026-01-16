use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub serial: SerialConfig,
    pub message: MessageConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: String,
    pub stop_bits: u8,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageConfig {
    pub shutdown_text: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            serial: SerialConfig {
                port: "COM1".to_string(),
                baud_rate: 9600,
                data_bits: 8,
                parity: "None".to_string(),
                stop_bits: 1,
            },
            message: MessageConfig {
                shutdown_text: "SHUTDOWN\r".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load(&path) {
            Ok(config) => {
                log::info!("Configuration loaded from {:?}", path.as_ref());
                config
            }
            Err(e) => {
                log::warn!("Failed to load configuration: {}. Using defaults.", e);
                let config = Config::default();
                if let Err(e) = config.save(&path) {
                    log::error!("Failed to save default configuration: {}", e);
                }
                config
            }
        }
    }
}
