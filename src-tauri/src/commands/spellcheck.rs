use std::path::PathBuf;
use std::time::Instant;
use tauri::{AppHandle, Manager};

use crate::engine::checker::CheckerEngine;
use crate::engine::dictionary::SpellEngine;
use crate::engine::docx_parser::DocxParser;
use crate::models::ScanResult;

// State holding our initialized engines
pub struct EngineState {
    pub checker: std::sync::Mutex<Option<CheckerEngine>>,
}

#[tauri::command]
pub async fn init_spellcheck(app: AppHandle) -> Result<String, String> {
    let app_dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    
    // Ensure dir exists
    let _ = std::fs::create_dir_all(&app_dir);
    
    // Attempt to load dictionaries
    let spell_engine = SpellEngine::new(&app_dir)?;
    let checker = CheckerEngine::new(spell_engine);
    
    let state = app.state::<EngineState>();
    *state.checker.lock().unwrap() = Some(checker);
    
    Ok("Spellcheck engines initialized successfully".to_string())
}

#[tauri::command]
pub async fn scan_document(
    app: AppHandle,
    file_path: String,
) -> Result<ScanResult, String> {
    let start = Instant::now();
    
    // Parse document
    let path = PathBuf::from(&file_path);
    let extracted = DocxParser::extract_text(&path)?;
    
    // Scan paragraphs
    let mut findings = Vec::new();
    
    let state = app.state::<EngineState>();
    let checker_guard = state.checker.lock().unwrap();
    
    let checker = checker_guard.as_ref().ok_or("Engine not initialized. Call init_spellcheck first.")?;
    
    for paragraph in &extracted.paragraphs {
        let p_findings = checker.scan_paragraph(paragraph);
        findings.extend(p_findings);
    }
    
    Ok(ScanResult {
        document: extracted.info,
        findings,
        total_words: extracted.paragraphs.iter().map(|p| p.text.split_whitespace().count()).sum(),
        scan_duration_ms: start.elapsed().as_millis() as u64,
    })
}
