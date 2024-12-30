// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use reqwest;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
// use tauri::command;
use tauri::Manager;
use tracing::{error, info};
mod commands;

#[tauri::command]
fn greet1(name: &str) -> String {
    println!("entering the greet when invoked");
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// #[tauri::command]
// pub async fn list_api_definitions(name: Option<String>) -> Result<String, String> {
//     println!("enteirgn this=======>");
//     let url = "http://localhost:9881/v1/api/definitions".to_string();

//     let client = reqwest::Client::new();
//     let response = client.get(&url).send().await.map_err(|e| e.to_string())?;

//     if response.status().is_success() {
//         Ok(response
//             .text()
//             .await
//             .unwrap_or_else(|_| "Success but empty response".to_string()))
//     } else {
//         Err(format!(
//             "Failed to list API definitions: {:?}",
//             response.text().await
//         ))
//     }
// }



#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet1, commands::list_api_definitions])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
