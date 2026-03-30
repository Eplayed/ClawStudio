// ClawStudio - main.rs
// Tauri application entry point with all command registrations

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audit;
mod computer_use;
mod db;
mod docker;
mod enterprise;
mod gateway;
mod keychain;
mod openclaw;
mod setup;
mod vnc_client;

use tauri::Manager;
use gateway::GatewayState;
use setup::SetupState;
use audit::AuditState;
use std::path::PathBuf;

fn main() {
    env_logger::init();
    
    // Get data directory for audit logs
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("clawstudio");
    std::fs::create_dir_all(&data_dir).ok();

    tauri::Builder::default()
        // -- Plugins --
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        // -- Managed State --
        .manage(openclaw::OpenClawManager::new())
        .manage(docker::DockerManager::new())
        .manage(computer_use::CURuntime::new())
        .manage(GatewayState::new())
        .manage(SetupState::new())
        .manage(AuditState::new(data_dir.join("audit.db")))
        // -- Commands --
        .invoke_handler(tauri::generate_handler![
            // Keychain
            keychain::save_api_key,
            keychain::get_api_key,
            keychain::delete_api_key,
            keychain::test_api_key,
            // OpenClaw process
            openclaw::start_agent,
            openclaw::stop_agent,
            openclaw::list_agents,
            openclaw::get_agent_status,
            // Docker sandbox
            docker::check_docker,
            docker::create_sandbox,
            docker::destroy_sandbox,
            docker::list_sandboxes,
            docker::get_sandbox_stats,
            // Computer Use
            computer_use::start_cu_session,
            computer_use::cu_step,
            computer_use::pause_cu_session,
            computer_use::resume_cu_session,
            computer_use::stop_cu_session,
            // VNC Client
            vnc_client::vnc_connect,
            vnc_client::vnc_screenshot,
            vnc_client::vnc_mouse_move,
            vnc_client::vnc_click,
            vnc_client::vnc_type_text,
            vnc_client::vnc_press_key,
            // Database helpers
            db::init_database,
            db::get_all_agents,
            db::save_agent_config,
            db::get_all_traces,
            db::save_trace,
            // Setup Wizard (Phase 1)
            setup::check_environment,
            setup::install_node,
            setup::install_openclaw,
            setup::configure_openclaw,
            setup::start_gateway_from_setup,
            setup::uninstall_openclaw,
            // Gateway Manager (Phase 2)
            gateway::start_gateway,
            gateway::stop_gateway,
            gateway::restart_gateway,
            gateway::gateway_health,
            gateway::gateway_status,
            gateway::gateway_logs,
            gateway::check_openclaw_update,
            gateway::upgrade_openclaw,
            // Audit & Traces (Phase 4)
            audit::log_audit_entry,
            audit::get_audit_logs,
            audit::export_audit_logs,
            audit::verify_audit_integrity,
            audit::get_cost_summary,
            audit::cleanup_old_audit_logs,
        ])
        // -- Setup hook --
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            let app_handle = app.handle().clone();

            // Initialize database tables on first launch
            tauri::async_runtime::spawn(async move {
                if let Err(e) = db::run_migrations(&app_handle).await {
                    log::error!("Database migration failed: {}", e);
                }
            });

            // Start heartbeat monitor for orphan OpenClaw processes
            let window_clone = window.clone();
            tauri::async_runtime::spawn(async move {
                openclaw::check_orphan_processes(window_clone).await;
            });

            log::info!("ClawStudio Nova v0.2 started");
            Ok(())
        })
        // -- Exit handler --
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // Cleanup Gateway process on exit
                let state = window.state::<GatewayState>();
                gateway::cleanup_on_exit(&state);
                log::info!("ClawStudio shutting down");
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running ClawStudio");
}