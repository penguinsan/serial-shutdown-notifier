use crate::config::Config;
use crate::serial::SerialSender;
use std::time::Duration;
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};

pub const SERVICE_NAME: &str = "SerialShutdownNotifier";
pub const SERVICE_DISPLAY_NAME: &str = "Serial Shutdown Notifier";

pub fn run_service() -> Result<(), windows_service::Error> {
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            ServiceControl::Stop | ServiceControl::Preshutdown => {
                log::info!("Received shutdown/stop event");

                // サービスディレクトリから設定ファイルを読み込む
                let exe_path = std::env::current_exe().unwrap_or_default();
                let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
                let config_path = exe_dir.join("config.toml");

                let config = Config::load_or_default(&config_path);
                let sender = SerialSender::new(config);

                match sender.send_shutdown_message() {
                    Ok(_) => log::info!("Shutdown message sent successfully"),
                    Err(e) => log::error!("Failed to send shutdown message: {}", e),
                }

                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // サービスが実行中であることを報告
    let running_status = ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::PRESHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    };

    status_handle.set_service_status(running_status)?;

    log::info!("Service is running");

    // サービスを実行し続ける（停止イベントまで待機）
    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}
