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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.serial.port, "COM1");
        assert_eq!(config.serial.baud_rate, 9600);
        assert_eq!(config.serial.data_bits, 8);
        assert_eq!(config.serial.parity, "None");
        assert_eq!(config.serial.stop_bits, 1);
        assert_eq!(config.message.shutdown_text, "SHUTDOWN\r");
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_file = "test_config.toml";

        // テスト用の設定を作成
        let config = Config {
            serial: SerialConfig {
                port: "COM3".to_string(),
                baud_rate: 115200,
                data_bits: 8,
                parity: "Even".to_string(),
                stop_bits: 1,
            },
            message: MessageConfig {
                shutdown_text: "TEST\r\n".to_string(),
            },
        };

        // 保存
        config.save(temp_file).expect("Failed to save config");

        // 読み込み
        let loaded_config = Config::load(temp_file).expect("Failed to load config");

        // 検証
        assert_eq!(loaded_config.serial.port, "COM3");
        assert_eq!(loaded_config.serial.baud_rate, 115200);
        assert_eq!(loaded_config.serial.parity, "Even");
        assert_eq!(loaded_config.message.shutdown_text, "TEST\r\n");

        // クリーンアップ
        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_load_or_default_with_nonexistent_file() {
        let temp_file = "nonexistent_config.toml";

        // 存在しないファイルを指定
        let config = Config::load_or_default(temp_file);

        // デフォルト値が返されることを確認
        assert_eq!(config.serial.port, "COM1");
        assert_eq!(config.serial.baud_rate, 9600);

        // クリーンアップ（load_or_defaultが作成したファイルを削除）
        let _ = fs::remove_file(temp_file);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).expect("Failed to serialize");

        // TOMLに必要なフィールドが含まれているか確認
        assert!(toml_str.contains("port"));
        assert!(toml_str.contains("baud_rate"));
        assert!(toml_str.contains("shutdown_text"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            [serial]
            port = "COM5"
            baud_rate = 57600
            data_bits = 7
            parity = "Odd"
            stop_bits = 2

            [message]
            shutdown_text = "HALT\r"
        "#;

        let config: Config = toml::from_str(toml_str).expect("Failed to deserialize");

        assert_eq!(config.serial.port, "COM5");
        assert_eq!(config.serial.baud_rate, 57600);
        assert_eq!(config.serial.data_bits, 7);
        assert_eq!(config.serial.parity, "Odd");
        assert_eq!(config.serial.stop_bits, 2);
        assert_eq!(config.message.shutdown_text, "HALT\r");
    }
}
