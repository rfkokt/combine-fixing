pub mod commands;
pub mod engine;
pub mod models;

use commands::spellcheck::{init_spellcheck, scan_document, EngineState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(EngineState {
            checker: std::sync::Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            init_spellcheck,
            scan_document
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
