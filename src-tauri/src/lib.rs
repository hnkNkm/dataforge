mod database;

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
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, test_database_connection])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
