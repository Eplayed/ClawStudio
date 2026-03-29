// ClawStudio - main.rs
// Tauri application entry point with all command registrations

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod computer_use;
mod db;
mod docker;
mod keychain;
mod openclaw;
mod vnc_client;

use tauri::Manager;

fn main() {
    env_logger::init();

    tauri::Builder::default()
        // -- Plugins --
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        // -- Managed State --
        .manage(openclaw::OpenClawManager::new())
        .manage(docker::DockerManager::new())
        .manage(computer_use::CURuntime::new())
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
        ])
        // -- Setup hook --
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            // Initialize database tables on first launch
            let app_handle = app.handle().clone();
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

            log::info!("ClawStudio Nova v0.1 started");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ClawStudio");
}
