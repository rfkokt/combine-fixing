use std::path::PathBuf;
use std::time::Instant;
use tauri::{AppHandle, Manager};

use crate::engine::checker::CheckerEngine;
use crate::engine::dictionary::SpellEngine;
use crate::engine::docx_parser::DocxParser;
use crate::models::{ScanResult, TypoFinding};

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
pub async fn load_document(
    file_path: String,
) -> Result<crate::models::DocumentInfo, String> {
    let path = PathBuf::from(&file_path);
    let extracted = tokio::task::spawn_blocking(move || {
        DocxParser::extract_text(&path)
    }).await.map_err(|e| e.to_string())??;
    
    Ok(extracted.info)
}

#[tauri::command]
pub async fn scan_document(
    app: AppHandle,
    file_path: String,
) -> Result<ScanResult, String> {
    let start = Instant::now();
    
    // Run CPU-heavy work in spawn_blocking
    let result = tokio::task::spawn_blocking(move || -> Result<ScanResult, String> {
        let path = PathBuf::from(&file_path);
        println!("Extracting text from {:?}", path);
        let extracted = DocxParser::extract_text(&path)?;
        
        let state = app.state::<EngineState>();
        let checker_guard = state.checker.lock().unwrap();
        let checker = checker_guard.as_ref().ok_or("Engine not initialized. Call init_spellcheck first.")?;
        
        println!("Scanning {} paragraphs...", extracted.paragraphs.len());
        let mut findings = Vec::new();
        let mut suggest_cache = std::collections::HashMap::new();
        
        for (i, paragraph) in extracted.paragraphs.iter().enumerate() {
            if i % 100 == 0 {
                println!("Scanned {}/{} paragraphs...", i, extracted.paragraphs.len());
            }
            let p_findings = checker.scan_paragraph(paragraph, &mut suggest_cache);
            findings.extend(p_findings);
        }
        
        println!("Scan complete! Found {} issues in {}ms", findings.len(), start.elapsed().as_millis());
        
        Ok(ScanResult {
            document: extracted.info,
            findings,
            total_words: extracted.paragraphs.iter().map(|p| p.text.split_whitespace().count()).sum(),
            scan_duration_ms: start.elapsed().as_millis() as u64,
        })
    }).await.map_err(|e| e.to_string())??;
    
    Ok(result)
}

#[tauri::command]
pub async fn export_document(
    input_path: String,
    output_path: String,
    findings: Vec<TypoFinding>,
) -> Result<String, String> {
    let in_path = PathBuf::from(&input_path);
    let out_path = PathBuf::from(&output_path);
    
    DocxParser::export_fixed_document(&in_path, &out_path, &findings)?;
    
    Ok("Document exported successfully".to_string())
}

#[tauri::command]
pub async fn save_file_copy(
    source_path: String,
    destination_path: String,
) -> Result<(), String> {
    std::fs::copy(&source_path, &destination_path)
        .map(|_| ())
        .map_err(|e| format!("Failed to copy file: {}", e))
}
