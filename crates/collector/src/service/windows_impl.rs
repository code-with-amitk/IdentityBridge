//! Windows Service integration using `windows-service`.

use std::ffi::OsString;
use std::sync::Arc;
use std::time::Duration;

use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
    ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::service_dispatcher;

use crate::config::CollectorConfig;
use crate::logging::component;

const SERVICE_NAME: &str = "IdentityBridgeCollector";
const SERVICE_DISPLAY_NAME: &str = "Collector";
const SERVICE_DESCRIPTION: &str =
    "Collects identity from Active Directory and pushes sessions to the Server.";

fn config_path_from_args(arguments: &[OsString]) -> String {
    let args: Vec<String> = arguments
        .iter()
        .filter_map(|a| a.to_str().map(String::from))
        .collect();

    for i in 0..args.len() {
        if args[i] == "--config" || args[i] == "-c" {
            if let Some(path) = args.get(i + 1) {
                return path.clone();
            }
        }
    }

    "configs/collector.yaml".into()
}

windows_service::define_windows_service!(ffi_service_main, service_main);

fn service_main(arguments: Vec<OsString>) {
    if let Err(e) = run_service(&arguments) {
        eprintln!("service failed: {e}");
    }
}

fn run_service(arguments: &[OsString]) -> anyhow::Result<()> {
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                let _ = shutdown_tx.send(());
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle =
        service_control_handler::register(SERVICE_NAME, event_handler)?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::StartPending,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(10),
        process_id: None,
    })?;

    let config_path = config_path_from_args(arguments);
    let config = Arc::new(CollectorConfig::from_file(&config_path)?);
    crate::logging::init_tracing(&config.logging)?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    tracing::info!(target: component::SERVICE, "Windows Service started");

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let runtime = Arc::new(crate::runtime::CollectorRuntime::new(config)?);
        runtime.spawn_background_tasks();
        tokio::select! {
            r = crate::http::run_http_server(runtime) => r,
            _ = wait_for_shutdown(shutdown_rx) => Ok(()),
        }
    })?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

async fn wait_for_shutdown(rx: std::sync::mpsc::Receiver<()>) {
    let _ = rx.recv();
    tracing::info!(target: component::SERVICE, "Windows Service stop requested");
}

/// Entry point when started by the Service Control Manager.
pub fn run(_config: Arc<CollectorConfig>) -> anyhow::Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

pub fn install_service(config_path: &str) -> anyhow::Result<()> {
    use std::path::PathBuf;
    use windows_service::service::ServiceAccess;
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    let manager =
        ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;

    let exe = std::env::current_exe()?;
    let service_binary_path = PathBuf::from(exe);

    let service_info = windows_service::service::ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: windows_service::service::ServiceStartType::AutoStart,
        error_control: windows_service::service::ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: vec![
            OsString::from("service"),
            OsString::from("--config"),
            OsString::from(config_path),
        ],
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };

    let service = manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;

    service.set_description(SERVICE_DESCRIPTION)?;

    tracing::info!(
        target: component::SERVICE,
        name = SERVICE_NAME,
        config_path,
        "Windows Service installed"
    );

    Ok(())
}

pub fn uninstall_service() -> anyhow::Result<()> {
    use windows_service::service::{ServiceAccess, ServiceState};
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};

    let manager =
        ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
    let service = manager.open_service(SERVICE_NAME, ServiceAccess::DELETE | ServiceAccess::STOP)?;

    let status = service.query_status()?;
    if status.current_state != ServiceState::Stopped {
        service.stop()?;
    }

    service.delete()?;
    tracing::info!(target: component::SERVICE, name = SERVICE_NAME, "Windows Service uninstalled");
    Ok(())
}
