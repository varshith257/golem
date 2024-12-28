// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use super::*;
fn main() {
    // golem_gui_lib::run()
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::upload_openapi_definition,
            commands::list_api_definitions,
            commands::create_api_definition,
            commands::update_api_definition,
            commands::delete_api_definition,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
