use crate::config::Config;
use serialport::{DataBits, Parity, SerialPort, StopBits};
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
    use crate::config::Config;

    #[test]
    fn test_serial_sender_creation() {
        let config = Config::default();
        let sender = SerialSender::new(config);
        assert_eq!(sender.config.serial.port, "COM1");
    }
}
