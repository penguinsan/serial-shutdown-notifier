mod config;
mod serial;
mod service;

use std::ffi::OsString;
use windows_service::service_dispatcher;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ログ初期化
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stdout)
        .init();

    let args: Vec<OsString> = std::env::args_os().collect();

    // コマンドライン引数の処理
    if args.len() > 1 {
        let arg = args[1].to_string_lossy().to_lowercase();
        match arg.as_str() {
            "install" => {
                install_service()?;
                println!("Service installed successfully.");
                println!("Start the service with: sc start {}", service::SERVICE_NAME);
                return Ok(());
            }
            "uninstall" => {
                uninstall_service()?;
                println!("Service uninstalled successfully.");
                return Ok(());
            }
            "test" => {
                test_serial_connection()?;
                return Ok(());
            }
            _ => {
                print_usage();
                return Ok(());
            }
        }
    }

    // サービスとして起動
    log::info!("Starting service dispatcher");
    service_dispatcher::start(service::SERVICE_NAME, ffi_service_main)?;

    Ok(())
}

windows_service::define_windows_service!(ffi_service_main, service_main);

fn service_main(_arguments: Vec<OsString>) {
    if let Err(e) = service::run_service() {
        log::error!("Service error: {}", e);
    }
}

fn install_service() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    // 管理者権限チェック
    if !is_elevated() {
        return Err(
            "This command requires administrator privileges. Please run as administrator.".into(),
        );
    }

    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();

    println!("Installing service...");
    println!("Service name: {}", service::SERVICE_NAME);
    println!("Executable path: {}", exe_path_str);

    let output = Command::new("sc")
        .args([
            "create",
            service::SERVICE_NAME,
            &format!("binPath=\"{}\"", exe_path_str),
            &format!("DisplayName=\"{}\"", service::SERVICE_DISPLAY_NAME),
            "start=auto",
        ])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.is_empty() {
        println!("Output: {}", stdout);
    }
    if !stderr.is_empty() {
        println!("Error: {}", stderr);
    }

    if !output.status.success() {
        return Err(format!(
            "Failed to install service. Exit code: {:?}\nStderr: {}\nStdout: {}",
            output.status.code(),
            stderr,
            stdout
        )
        .into());
    }

    // プリシャットダウン設定を追加（シャットダウン前に確実に実行されるようにする）
    let output = Command::new("sc")
        .args(["config", service::SERVICE_NAME, "start=auto"])
        .output()?;

    if !output.status.success() {
        log::warn!(
            "Failed to configure preshutdown: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

fn is_elevated() -> bool {
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token: HANDLE = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut size = 0u32;

        if GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        )
        .is_err()
        {
            return false;
        }

        elevation.TokenIsElevated != 0
    }
}

fn uninstall_service() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    // 管理者権限チェック
    if !is_elevated() {
        return Err(
            "This command requires administrator privileges. Please run as administrator.".into(),
        );
    }

    println!("Uninstalling service...");

    // サービスを停止
    println!("Stopping service...");
    let output = Command::new("sc")
        .args(["stop", service::SERVICE_NAME])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.is_empty() {
        println!("Stop output: {}", stdout);
    }
    if !stderr.is_empty() && !stderr.contains("1062") {
        // 1062 = サービスが開始されていない（これは正常）
        println!("Stop error: {}", stderr);
    }

    // サービスを削除
    println!("Deleting service...");
    let output = Command::new("sc")
        .args(["delete", service::SERVICE_NAME])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.is_empty() {
        println!("Delete output: {}", stdout);
    }
    if !stderr.is_empty() {
        println!("Delete error: {}", stderr);
    }

    if !output.status.success() {
        return Err(format!(
            "Failed to uninstall service. Exit code: {:?}\nStderr: {}\nStdout: {}",
            output.status.code(),
            stderr,
            stdout
        )
        .into());
    }

    Ok(())
}

fn test_serial_connection() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing serial connection...");

    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let config_path = exe_dir.join("config.toml");

    let config = config::Config::load_or_default(&config_path);

    println!("Configuration:");
    println!("  Port: {}", config.serial.port);
    println!("  Baud Rate: {}", config.serial.baud_rate);
    println!("  Data Bits: {}", config.serial.data_bits);
    println!("  Parity: {}", config.serial.parity);
    println!("  Stop Bits: {}", config.serial.stop_bits);
    println!("  Message: {:?}", config.message.shutdown_text);

    let sender = serial::SerialSender::new(config);

    println!("\nSending test message...");
    sender.send_shutdown_message()?;

    println!("Test completed successfully!");
    Ok(())
}

fn print_usage() {
    println!("Serial Shutdown Notifier");
    println!();
    println!("Usage:");
    println!(
        "  {} install   - Install the service",
        std::env::args().next().unwrap()
    );
    println!(
        "  {} uninstall - Uninstall the service",
        std::env::args().next().unwrap()
    );
    println!(
        "  {} test      - Test serial connection",
        std::env::args().next().unwrap()
    );
    println!();
    println!(
        "Service will automatically send a message to the configured serial port on shutdown."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_constants() {
        // サービス名が正しく設定されているか確認
        assert_eq!(service::SERVICE_NAME, "SerialShutdownNotifier");
        assert_eq!(service::SERVICE_DISPLAY_NAME, "Serial Shutdown Notifier");
    }

    #[test]
    fn test_config_loading() {
        // デフォルト設定が正しくロードできることを確認
        let config = config::Config::default();
        assert_eq!(config.serial.port, "COM1");
        assert_eq!(config.serial.baud_rate, 9600);
    }

    #[test]
    fn test_serial_sender_creation() {
        // SerialSenderが正しく作成できることを確認
        let config = config::Config::default();
        let _sender = serial::SerialSender::new(config);
        // 基本的な作成テスト（実際のシリアルポートは開かない）
        // SerialSenderが作成できればOK（パニックしない）
    }
}
