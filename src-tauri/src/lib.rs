pub mod commands;
pub mod engine;
pub mod models;

use commands::spellcheck::{init_spellcheck, load_document, scan_document, export_document, save_file_copy, EngineState};
use commands::ai::fix_document_with_ai;
use commands::merge::{merge_documents, get_document_preview, validate_docx};

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
            load_document,
            scan_document,
            export_document,
            fix_document_with_ai,
            save_file_copy,
            merge_documents,
            get_document_preview,
            validate_docx
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
