use std::io::Read;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use serde_json::json;
use crate::engine::merger::{Merger, SeparatorType};

#[derive(serde::Serialize)]
pub struct MergeResult {
    pub output_path: String,
    pub total_paragraphs: usize,
    pub total_words: usize,
    pub documents_merged: usize,
}

#[derive(serde::Serialize)]
pub struct DocumentPreview {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub preview_text: String,
    pub word_count: usize,
    pub paragraph_count: usize,
}

fn emit_merge_progress(app: &AppHandle, current: usize, total: usize, status: &str, message: &str) {
    let percentage = if total > 0 { (current as f64 / total as f64 * 100.0) as u32 } else { 0 };
    app.emit("merge-progress", json!({
        "current": current,
        "total": total,
        "percentage": percentage,
        "status": status,
        "message": message
    })).ok();
}

#[tauri::command]
pub async fn merge_documents(
    app: AppHandle,
    input_paths: Vec<String>,
    output_path: String,
    separator: String,
) -> Result<MergeResult, String> {
    let sep_type = SeparatorType::from_str(&separator);
    let out_path = PathBuf::from(&output_path);

    emit_merge_progress(&app, 0, input_paths.len(), "preparing",
        &format!("Preparing to merge {} documents...", input_paths.len()));

    let result = Merger::merge_documents(&input_paths, &out_path, sep_type)?;

    emit_merge_progress(&app, input_paths.len(), input_paths.len(), "done",
        &format!("Merged {} documents successfully!", result.documents_merged));

    Ok(MergeResult {
        output_path: result.output_path.to_string_lossy().to_string(),
        total_paragraphs: result.total_paragraphs,
        total_words: result.total_words,
        documents_merged: result.documents_merged,
    })
}

#[tauri::command]
pub async fn get_document_preview(file_path: String) -> Result<DocumentPreview, String> {
    let path = PathBuf::from(&file_path);

    let metadata = std::fs::metadata(&path)
        .map_err(|e| format!("Failed to read file metadata: {}", e))?;

    let preview_text = Merger::get_document_preview(&path)?;

    let file_name = path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let file = std::fs::File::open(&path).map_err(|e| format!("Failed to open: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read archive: {}", e))?;

    let mut document_xml = String::new();
    let mut word_count = 0;
    let mut paragraph_count = 0;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("Failed to read: {}", e))?;
        if file.name() == "word/document.xml" {
            file.read_to_string(&mut document_xml).map_err(|e| format!("Failed to read xml: {}", e))?;
        }
    }

    if !document_xml.is_empty() {
        let text_regex = regex::Regex::new(r"<w:t[^>]*>([^<]*)</w:t>").unwrap();
        for cap in text_regex.captures_iter(&document_xml) {
            if let Some(words) = cap.get(1) {
                word_count += words.as_str().split_whitespace().count();
            }
        }
        let para_regex = regex::Regex::new(r"<w:p[>\s]").unwrap();
        paragraph_count = para_regex.find_iter(&document_xml).count();
    }

    Ok(DocumentPreview {
        path: file_path,
        name: file_name,
        size: metadata.len(),
        preview_text,
        word_count,
        paragraph_count,
    })
}

#[tauri::command]
pub fn validate_docx(file_path: String) -> Result<bool, String> {
    let path = PathBuf::from(&file_path);

    let file = std::fs::File::open(&path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Invalid or corrupted docx file: {}", e))?;

    let mut has_document_xml = false;
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            if file.name() == "word/document.xml" {
                has_document_xml = true;
                break;
            }
        }
    }

    if !has_document_xml {
        return Err("File is not a valid .docx document".to_string());
    }

    Ok(true)
}