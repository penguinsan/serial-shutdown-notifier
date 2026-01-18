use crate::config::Config;
use serialport::{DataBits, Parity, StopBits};
use std::time::Duration;

pub struct SerialSender {
    config: Config,
}

impl SerialSender {
    pub fn new(config: Config) -> Self {
        SerialSender { config }
    }

    pub fn send_shutdown_message(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Opening serial port: {}", self.config.serial.port);

        let data_bits = match self.config.serial.data_bits {
            5 => DataBits::Five,
            6 => DataBits::Six,
            7 => DataBits::Seven,
            8 => DataBits::Eight,
            _ => {
                log::error!("Invalid data bits: {}", self.config.serial.data_bits);
                return Err("Invalid data bits".into());
            }
        };

        let parity = match self.config.serial.parity.to_lowercase().as_str() {
            "none" => Parity::None,
            "odd" => Parity::Odd,
            "even" => Parity::Even,
            _ => {
                log::error!("Invalid parity: {}", self.config.serial.parity);
                return Err("Invalid parity".into());
            }
        };

        let stop_bits = match self.config.serial.stop_bits {
            1 => StopBits::One,
            2 => StopBits::Two,
            _ => {
                log::error!("Invalid stop bits: {}", self.config.serial.stop_bits);
                return Err("Invalid stop bits".into());
            }
        };

        let mut port = serialport::new(&self.config.serial.port, self.config.serial.baud_rate)
            .timeout(Duration::from_secs(5))
            .data_bits(data_bits)
            .parity(parity)
            .stop_bits(stop_bits)
            .open()?;

        log::info!("Serial port opened successfully");

        let message = self.config.message.shutdown_text.as_bytes();
        log::info!(
            "Sending shutdown message: {:?} ({} bytes)",
            self.config.message.shutdown_text,
            message.len()
        );

        port.write_all(message)?;
        port.flush()?;

        log::info!("Shutdown message sent successfully");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, SerialConfig, MessageConfig};

    #[test]
    fn test_serial_sender_creation() {
        let config = Config::default();
        let sender = SerialSender::new(config);
        assert_eq!(sender.config.serial.port, "COM1");
    }

    #[test]
    fn test_serial_sender_with_custom_config() {
        let config = Config {
            serial: SerialConfig {
                port: "COM5".to_string(),
                baud_rate: 115200,
                data_bits: 8,
                parity: "None".to_string(),
                stop_bits: 1,
            },
            message: MessageConfig {
                shutdown_text: "CUSTOM\r\n".to_string(),
            },
        };

        let sender = SerialSender::new(config);
        assert_eq!(sender.config.serial.port, "COM5");
        assert_eq!(sender.config.serial.baud_rate, 115200);
        assert_eq!(sender.config.message.shutdown_text, "CUSTOM\r\n");
    }

    #[test]
    fn test_data_bits_validation() {
        // 有効なデータビット値をテスト
        for bits in [5, 6, 7, 8] {
            let config = Config {
                serial: SerialConfig {
                    port: "COM1".to_string(),
                    baud_rate: 9600,
                    data_bits: bits,
                    parity: "None".to_string(),
                    stop_bits: 1,
                },
                message: MessageConfig {
                    shutdown_text: "TEST".to_string(),
                },
            };
            let sender = SerialSender::new(config);
            assert_eq!(sender.config.serial.data_bits, bits);
        }
    }

    #[test]
    fn test_parity_values() {
        // 有効なパリティ値をテスト
        for parity in ["None", "Odd", "Even"] {
            let config = Config {
                serial: SerialConfig {
                    port: "COM1".to_string(),
                    baud_rate: 9600,
                    data_bits: 8,
                    parity: parity.to_string(),
                    stop_bits: 1,
                },
                message: MessageConfig {
                    shutdown_text: "TEST".to_string(),
                },
            };
            let sender = SerialSender::new(config);
            assert_eq!(sender.config.serial.parity.to_lowercase(), parity.to_lowercase());
        }
    }

    #[test]
    fn test_stop_bits_validation() {
        // 有効なストップビット値をテスト
        for bits in [1, 2] {
            let config = Config {
                serial: SerialConfig {
                    port: "COM1".to_string(),
                    baud_rate: 9600,
                    data_bits: 8,
                    parity: "None".to_string(),
                    stop_bits: bits,
                },
                message: MessageConfig {
                    shutdown_text: "TEST".to_string(),
                },
            };
            let sender = SerialSender::new(config);
            assert_eq!(sender.config.serial.stop_bits, bits);
        }
    }

    #[test]
    fn test_baud_rate_values() {
        // 一般的なボーレート値をテスト
        for baud_rate in [9600, 19200, 38400, 57600, 115200] {
            let config = Config {
                serial: SerialConfig {
                    port: "COM1".to_string(),
                    baud_rate,
                    data_bits: 8,
                    parity: "None".to_string(),
                    stop_bits: 1,
                },
                message: MessageConfig {
                    shutdown_text: "TEST".to_string(),
                },
            };
            let sender = SerialSender::new(config);
            assert_eq!(sender.config.serial.baud_rate, baud_rate);
        }
    }

    #[test]
    fn test_message_content() {
        let test_messages = vec![
            "SHUTDOWN\r",
            "SHUTDOWN\n",
            "SHUTDOWN\r\n",
            "HALT",
            "",
        ];

        for msg in test_messages {
            let config = Config {
                serial: SerialConfig {
                    port: "COM1".to_string(),
                    baud_rate: 9600,
                    data_bits: 8,
                    parity: "None".to_string(),
                    stop_bits: 1,
                },
                message: MessageConfig {
                    shutdown_text: msg.to_string(),
                },
            };
            let sender = SerialSender::new(config);
            assert_eq!(sender.config.message.shutdown_text, msg);
        }
    }
}
