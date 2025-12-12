// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use bubble_core::BubbleClient;
use bubble_core::nostr_sdk::prelude::*;
use tauri::State;
use tauri::Manager;
use tokio::sync::Mutex;
use std::sync::Arc;

struct AppState {
    client: Arc<Mutex<Option<BubbleClient>>>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_timeline(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let client_guard = state.client.lock().await;
    if let Some(client) = client_guard.as_ref() {
        match client.get_timeline(20).await {
            Ok(events) => {
                // Serialize events to JSON strings for simplicity in frontend
                Ok(events.iter().map(|e| serde_json::to_string(e).unwrap_or_default()).collect())
            },
            Err(e) => Err(e.to_string()),
        }
    } else {
        Err("Client not initialized".to_string())
    }
}

#[tauri::command]
async fn publish_note(content: String, state: State<'_, AppState>) -> Result<String, String> {
    let client_guard = state.client.lock().await;
    if let Some(client) = client_guard.as_ref() {
        match client.publish_text_note(&content).await {
            Ok(event_id) => Ok(event_id.to_string()),
            Err(e) => Err(e.to_string()),
        }
    } else {
         Err("Client not initialized".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState { client: Arc::new(Mutex::new(None)) })
        .setup(|app| {
             let app_handle = app.handle().clone();
             tauri::async_runtime::spawn(async move {
                 // Initialize client in background
                 match BubbleClient::new(None).await {
                     Ok(client) => {
                         let state = app_handle.state::<AppState>();
                         *state.client.lock().await = Some(client);
                         println!("Bubble Client initialized!");
                     },
                     Err(e) => {
                         eprintln!("Failed to initialize Bubble Client: {}", e);
                     }
                 }
             });
             Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, get_timeline, publish_note])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
