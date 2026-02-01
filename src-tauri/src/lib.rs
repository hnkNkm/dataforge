mod commands;
mod database;
mod error;
mod logger;
mod profile;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn test_database_connection() -> Result<String, String> {
    use database::DatabaseConnection;

    // Load .env file
    dotenv::dotenv().ok();

    match DatabaseConnection::from_env().await {
        Ok(conn) => {
            match conn.test_connection().await {
                Ok(true) => {
                    let version = conn.version().await.unwrap_or_else(|_| "Unknown".to_string());
                    let db_name = conn.current_database().await.unwrap_or_else(|_| "Unknown".to_string());
                    conn.close().await;
                    Ok(format!("✅ Connected to PostgreSQL\nDatabase: {}\nVersion: {}", db_name, version))
                },
                Ok(false) => Err("Connection test failed".to_string()),
                Err(e) => Err(format!("Connection test error: {}", e))
            }
        },
        Err(e) => Err(format!("Failed to connect: {}", e))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logger
    let log_level = if cfg!(debug_assertions) {
        logger::LogLevel::Debug
    } else {
        logger::LogLevel::Info
    };

    // Get app data directory for log file
    let log_file = if let Ok(home_dir) = std::env::var("HOME") {
        let log_dir = std::path::PathBuf::from(home_dir)
            .join(".dataforge")
            .join("logs");
        std::fs::create_dir_all(&log_dir).ok();
        Some(log_dir.join("dataforge.log"))
    } else {
        None
    };

    // Initialize the logger
    if let Err(e) = logger::init_logger(log_level, log_file) {
        eprintln!("Failed to initialize logger: {}", e);
    }

    log_info!("main", "Starting DataForge application");

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(commands::profile::ProfileManagerState::new());
    
    // Add MCP Bridge plugin for development builds
    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
        log_info!("main", "MCP Bridge plugin enabled for development");
    }
    
    builder
        .invoke_handler(tauri::generate_handler![
            greet,
            test_database_connection,
            commands::connect_database,
            commands::disconnect_database,
            commands::test_database_connection_adapter,
            commands::execute_query,
            commands::get_database_metadata,
            commands::list_database_tables,
            commands::cancel_connection,
            commands::get_table_indexes,
            commands::generate_select_query,
            commands::profile::create_profile,
            commands::profile::list_profiles,
            commands::profile::get_profile,
            commands::profile::update_profile,
            commands::profile::delete_profile,
            commands::profile::connect_with_profile,
        ])
        .setup(|_app| {
            log_info!("main", "Application setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
