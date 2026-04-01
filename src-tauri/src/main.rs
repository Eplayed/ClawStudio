// ClawStudio - main.rs
// Tauri application entry point with all command registrations

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Import from lib
use clawstudio_lib::*;

use tauri::Manager;
use gateway::GatewayState;
use setup::SetupState;
use audit::AuditManager;
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
        .plugin(tauri_plugin_shell::init())
        // -- Managed State --
        .manage(openclaw::OpenClawManager::new())
        .manage(docker::DockerManager::new())
        .manage(computer_use::CURuntime::new())
        .manage(GatewayState::new())
        .manage(SetupState::new())
        .manage(AuditManager::new())
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
            // Setup Wizard
            setup::check_environment,
            setup::get_env_status,
            setup::install_node,
            setup::install_openclaw,
            setup::configure_openclaw,
            setup::start_gateway_from_setup,
            setup::uninstall_openclaw,
            // Gateway Manager
            gateway::start_gateway,
            gateway::stop_gateway,
            gateway::restart_gateway,
            gateway::gateway_health,
            gateway::gateway_status,
            gateway::gateway_logs,
            gateway::check_openclaw_update,
            gateway::upgrade_openclaw,
            // Channel Management
            channels::list_channels,
            channels::add_channel,
            channels::remove_channel,
            channels::test_channel,
            channels::restart_gateway_for_channels,
            // Audit & Traces
            audit::audit_log,
            audit::audit_get_entries,
            audit::audit_get_stats,
            audit::audit_export,
            audit::audit_cleanup,
            audit::get_cost_summary,
            audit::export_audit_logs,
            audit::get_audit_logs,
            // Template Management
            template::export_template,
            template::export_template_file,
            template::import_template,
            template::import_template_file,
            template::generate_share_link,
            template::validate_template,
            template::list_builtin_templates,
            template::get_builtin_template,
        ])
        // -- Setup hook --
        .setup(|_app| {
            log::info!("ClawStudio Nova v0.2 started");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ClawStudio");
}
