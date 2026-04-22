use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use reqwest::Client;
use futures::stream::{self, StreamExt};
use tokio::sync::Mutex;
use regex::Regex;

use crate::engine::docx_parser::DocxParser;
use crate::models::{DocumentParagraph, TypoFinding, TypoSeverity, TypoSource, TypoStatus, TypoPosition};
use super::spellcheck::EngineState;

// ────────────────────────────────────────────────────────────────
// Progress Emitter
// ────────────────────────────────────────────────────────────────

fn emit_progress(app: &AppHandle, current: usize, total: usize, status: &str, fixed_count: usize, message: &str) {
    let percentage = if total > 0 { (current as f64 / total as f64 * 100.0) as u32 } else { 0 };
    app.emit("ai-progress", json!({
        "current": current,
        "total": total,
        "percentage": percentage,
        "status": status,
        "fixedCount": fixed_count,
        "message": message
    })).ok();
}

// ────────────────────────────────────────────────────────────────
// Stage 1: Auto-fix (Regex-based, no AI)
// ────────────────────────────────────────────────────────────────

/// Apply comprehensive regex-based fixes to all paragraphs.
/// Handles common Indonesian patterns that AI often misses.
/// This is the FREE stage - no API calls needed.
fn auto_fix_paragraphs(paragraphs: &[DocumentParagraph]) -> HashMap<usize, String> {
    let mut fixes = HashMap::new();

    // Skip paragraphs that contain Word field codes (TOC, page numbers, etc.)
    let field_code_re = Regex::new(r"\\|TOC\s|PAGEN| PAGE |NUMPAGES").unwrap();

    // Skip paragraphs that look like TOC entries
    let toc_dot_leader_re = Regex::new(r"\.\.+\s+\d+$|\.\.+\s+$").unwrap();

    // ========================
    // ALL REPLACEMENT PATTERNS
    // ========================
    // Format: (pattern, replacement)
    let all_replacements: Vec<(Regex, &str)> = vec![
        // ========================
        // SPELLING ERRORS (from analysis)
        // ========================
        (Regex::new(r"puskemas").unwrap(), "Puskesmas"),
        (Regex::new(r"bedasarkan").unwrap(), "berdasarkan"),
        (Regex::new(r"layaanan").unwrap(), "layanan"),
        (Regex::new(r"deksripsi").unwrap(), "deskripsi"),
        (Regex::new(r"menyakjian").unwrap(), "menyajikan"),
        (Regex::new(r"dijalankann").unwrap(), "dijalankan"),
        (Regex::new(r"perencaan").unwrap(), "perencanaan"),
        (Regex::new(r"managemen").unwrap(), "Manajemen"),
        (Regex::new(r"control").unwrap(), "kontrol"),
        (Regex::new(r"penomeran").unwrap(), "penomoran"),
        (Regex::new(r"surveior").unwrap(), "surveyor"),
        (Regex::new(r"tetang").unwrap(), "tentang"),
        (Regex::new(r"tuugas").unwrap(), "tugas"),
        (Regex::new(r"jabalan").unwrap(), "jabatan"),
        (Regex::new(r"beisi").unwrap(), "berisi"),
        (Regex::new(r"melibat").unwrap(), "melibatkan"),
        (Regex::new(r"fasilit as").unwrap(), "fasilitas"),
        (Regex::new(r"fasilitass").unwrap(), "fasilitas"), // extra s
        (Regex::new(r"permusnahan").unwrap(), "pemusnahan"),
        (Regex::new(r"dikendallikan").unwrap(), "dikendalikan"),
        (Regex::new(r"kualifi kasi").unwrap(), "Kualifikasi"),
        (Regex::new(r"teridentifi kasi").unwrap(), "teridentifikasi"),
        (Regex::new(r"paying").unwrap(), "payung"),
        (Regex::new(r"pijakamatau").unwrap(), "pijakamatau"), // tricky

        // ========================
        // WORD BLENDS - Common Suffixes (Indonesian)
        // ========================
        // Format: find XxxxYyyy where Yyyy is common word
        (Regex::new(r"yangakan").unwrap(), "yang akan"),
        (Regex::new(r"yangdan").unwrap(), "yang dan"),
        (Regex::new(r"yanguntuk").unwrap(), "yang untuk"),
        (Regex::new(r"yangdengan").unwrap(), "yang dengan"),
        (Regex::new(r"yangmemerlukan").unwrap(), "yang memerlukan"),
        (Regex::new(r"dengandan").unwrap(), "dengan dan"),
        (Regex::new(r"denganuntuk").unwrap(), "dengan untuk"),
        (Regex::new(r"untukdan").unwrap(), "untuk dan"),
        (Regex::new(r"untukmenyelesaikan").unwrap(), "untuk menyelesaikan"),
        (Regex::new(r"volumedengan").unwrap(), "volume dengan"),
        (Regex::new(r"tahunanuntuk").unwrap(), "tahunan untuk"),
        (Regex::new(r"pendukungdan").unwrap(), "pendukung dan"),
        (Regex::new(r"kegiatanyangakan").unwrap(), "kegiatan yang akan"),
        (Regex::new(r"kegiatanbaruyang").unwrap(), "kegiatan baru yang"),
        (Regex::new(r"masyarakatakan").unwrap(), "masyarakat akan"),
        (Regex::new(r"lintassektoral").unwrap(), "lintas sektoral"),
        (Regex::new(r"analisiskesehatan").unwrap(), "analisis kesehatan"),
        (Regex::new(r"pengembanganssecara").unwrap(), "pengembangansecara"), // will be fixed below
        (Regex::new(r"pengembangansecara").unwrap(), "pengembangan secara"),
        (Regex::new(r"pengembanganssecara").unwrap(), "pengembangan secara"),
        (Regex::new(r"tahunsebelumnya").unwrap(), "tahun sebelumnya"),
        (Regex::new(r"terangkumdalamusulan").unwrap(), "terangkum dalamusulan"), // will be fixed below
        (Regex::new(r"terangkum dalamusulan").unwrap(), "terangkum dalam mengusulkan"),
        (Regex::new(r"terangkumdalamusulan").unwrap(), "terangkum dalam usulan"),
        (Regex::new(r"pembiayaandandukungan").unwrap(), "pembiayaan dandukungan"),
        (Regex::new(r"pembiayaandan").unwrap(), "pembiayaan dan"),
        (Regex::new(r"baiksecara").unwrap(), "baik secara"),
        (Regex::new(r"usulanpembiayaan").unwrap(), "usulan pembiayaan"),
        (Regex::new(r"prasaranadan").unwrap(), "prasarana dan"),
        (Regex::new(r"telahditetapkan").unwrap(), "telah ditetapkan"),
        (Regex::new(r"protokolklinis").unwrap(), "protokol klinis"),
        (Regex::new(r"disampaikanke").unwrap(), "disampaikan ke"),
        (Regex::new(r"maupunpenulisan").unwrap(), "maupun penulisan"),
        (Regex::new(r"unitupaya").unwrap(), "unit upaya"),
        (Regex::new(r"FKTPatau").unwrap(), "FKTP atau"),
        (Regex::new(r"UKMdengan").unwrap(), "UKM dengan"),
        (Regex::new(r"perubahandokumen").unwrap(), "perubahan dokumen"),
        (Regex::new(r"harusdapat").unwrap(), "harus dapat"),
        (Regex::new(r"Penyimpanandokumen").unwrap(), "Penyimpanan dokumen"),
        (Regex::new(r"Pendistribusiandokumen").unwrap(), "Pendistribusian dokumen"),
        (Regex::new(r"PenyusunanPerubahanDokumen").unwrap(), "Penyusunan/Perubahan Dokumen"),
        (Regex::new(r"PengendaliDokumen").unwrap(), "Pengendali Dokumen"),
        (Regex::new(r"penomoranDokumen").unwrap(), "penomoran Dokumen"),
        (Regex::new(r"diberinomor").unwrap(), "diberi nomor"),
        (Regex::new(r"FKTPagarmembuatkebijakan").unwrap(), "FKTPagar membuat kebijakan"),
        (Regex::new(r"tatanaskah").unwrap(), "tata naskah"),
        (Regex::new(r"cekulang").unwrap(), "cek ulang"),
        (Regex::new(r"menjadidua").unwrap(), "menjadi dua"),
        (Regex::new(r"yang harusdilakukan").unwrap(), "yang harus dilakukan"),
        (Regex::new(r"harusdilakukan").unwrap(), "harus dilakukan"),
        (Regex::new(r"tilikesuaidengan").unwrap(), "tilik sesuai dengan"),
        (Regex::new(r"perbaikan/revisiisi").unwrap(), "perbaikan/revisi isi"),

        // ========================
        // MORE WORD BLENDS
        // ========================
        // Pijakamatau variations
        (Regex::new(r"pijakamatau").unwrap(), "pijakam atau"),
        (Regex::new(r"pijakam atau").unwrap(), "pijakam atau"),
        // Numbers + words (digit followed by letters)
        (Regex::new(r"(\d)([a-zA-Z]\w*)").unwrap(), "$1 $2"),  // "2tahun" -> "2 tahun"

        // English blends
        (Regex::new(r"healthanalysis").unwrap(), "health analysis"),
        // Revisi blends
        (Regex::new(r"revisiisi").unwrap(), "revisi isi"),
        (Regex::new(r"perbaikan/revisi").unwrap(), "perbaikan/revisi"),
        // Common extra spaces
        (Regex::new(r"pengesahan").unwrap(), "pengesahan"), // already correct

        // ========================
        // JATINEGARA BLENDS
        // ========================
        (Regex::new(r"Jatinegaradalam").unwrap(), "Jatinegara dalam"),
        (Regex::new(r"Jatinegaraini").unwrap(), "Jatinegara ini"),
        (Regex::new(r"Jatinegaraber").unwrap(), "Jatinegara ber"),
        (Regex::new(r"Jatinegarauntuk").unwrap(), "Jatinegara untuk"),
        (Regex::new(r"Jatinegarayang").unwrap(), "Jatinegara yang"),
        (Regex::new(r"Jatinegaradengan").unwrap(), "Jatinegara dengan"),
        (Regex::new(r"Jatinegaradan").unwrap(), "Jatinegara dan"),
        (Regex::new(r"Jatinegaraharus").unwrap(), "Jatinegara harus"),
        (Regex::new(r"Jatinegarabisa").unwrap(), "Jatinegara bisa"),
        (Regex::new(r"Jatinegarajuga").unwrap(), "Jatinegara juga"),
        (Regex::new(r"Jatinegarater").unwrap(), "Jatinegara ter"),
        (Regex::new(r"Jatinegaralebar").unwrap(), "Jatinegara lebar"),
        (Regex::new(r"Jatinegaradiharapkan").unwrap(), "Jatinegara diharapkan"),
        (Regex::new(r"Jatinegaraber").unwrap(), "Jatinegara ber"),
        (Regex::new(r"JatinegaraJatinegara").unwrap(), "Jatinegara Jatinegara"),

        // ========================
        // CAPITALIZED BLENDS
        // ========================
        (Regex::new(r"KesehatanMasyarakat").unwrap(), "Kesehatan Masyarakat"),
        (Regex::new(r"WaktuPelaksanaan").unwrap(), "Waktu Pelaksanaan"),
        (Regex::new(r"Kabupaten/Kota.Banyak").unwrap(), "Kabupaten/Kota. Banyak"),
        (Regex::new(r"Kabupaten/Kota\.").unwrap(), "Kabupaten/Kota. "),

        // ========================
        // PUNCTUATION + SPACE FIXES
        // ========================
        // Missing space after comma/period
        (Regex::new(r",([a-zA-Z])").unwrap(), ", $1"),
        (Regex::new(r"\.([A-Za-z])").unwrap(), ". $1"),
        // Double semicolons
        (Regex::new(r";;+").unwrap(), ";"),
        // Double spaces
        (Regex::new(r"  +").unwrap(), " "),
        // Double commas
        (Regex::new(r",,").unwrap(), ","),
        // Triple dots
        (Regex::new(r"\.{3,}").unwrap(), "..."),
    ];

    for p in paragraphs {
        let text = &p.text;

        // Skip paragraphs with field codes
        if field_code_re.is_match(text) {
            continue;
        }

        // Skip TOC-like entries
        if toc_dot_leader_re.is_match(text) {
            continue;
        }

        // Skip very short paragraphs
        if text.len() < 5 {
            continue;
        }

        let mut fixed = text.clone();

        // Apply ALL replacements with case-insensitive matching
        for _ in 0..5 {
            let before = fixed.clone();
            for (pattern, replacement) in all_replacements.iter() {
                // Make pattern case-insensitive
                let pattern_str = pattern.as_str();
                let case_insensitive_pattern = format!("(?i){}", pattern_str);
                if let Ok(re) = Regex::new(&case_insensitive_pattern) {
                    fixed = re.replace_all(&fixed, *replacement).to_string();
                }
            }
            if fixed == before {
                break;
            }
        }

        // Final cleanup: ensure space after punctuation
        fixed = Regex::new(r",([A-Za-z])").unwrap().replace_all(&fixed, ", $1").to_string();
        fixed = Regex::new(r"\.([A-Za-z])").unwrap().replace_all(&fixed, ". $1").to_string();

        // Final trim
        let trimmed = fixed.trim().to_string();
        if trimmed != fixed {
            fixed = trimmed;
        }

        if fixed != *text {
            fixes.insert(p.index, fixed);
        }
    }

    fixes
}

// ────────────────────────────────────────────────────────────────
// Smart Batching (character-based)
// ────────────────────────────────────────────────────────────────

fn build_smart_batches(paragraphs: Vec<DocumentParagraph>) -> Vec<Vec<DocumentParagraph>> {
    let max_chars_per_batch = 4000;
    let max_paragraphs_per_batch = 50;

    let mut batches: Vec<Vec<DocumentParagraph>> = Vec::new();
    let mut current_batch = Vec::new();
    let mut current_chars = 0;

    for p in paragraphs {
        if (current_chars + p.text.len() > max_chars_per_batch || current_batch.len() >= max_paragraphs_per_batch) 
            && !current_batch.is_empty() 
        {
            batches.push(current_batch);
            current_batch = Vec::new();
            current_chars = 0;
        }
        current_chars += p.text.len();
        current_batch.push(p);
    }
    if !current_batch.is_empty() {
        batches.push(current_batch);
    }

    batches
}

// ────────────────────────────────────────────────────────────────
// Process single AI batch
// ────────────────────────────────────────────────────────────────

async fn process_batch(
    client: &Client,
    endpoint: &str,
    api_key: &str,
    model: &str,
    batch: &[DocumentParagraph],
    _batch_idx: usize,
) -> Result<Vec<TypoFinding>, String> {
    let mut prompt_input = String::new();
    for p in batch {
        prompt_input.push_str(&format!("--- ID: {} ---\n{}\n\n", p.index, p.text));
    }

    let prompt = format!(
        r#"Kamu adalah EDITOR DOKUMEN PROFESIONAL Bahasa Indonesia. Tugas utamamu adalah memperbaiki kesalahan penulisan.

PERATURAN WAJIB (jangan lewatkan!):
1. Pisahkan kata-kata yang menempel tanpa spasi:
   - "pengembangansecara" → "pengembangan secara"
   - "analisiskesehatan" → "analisis kesehatan"
   - "kegiatanyangakan" → "kegiatan yang akan"
   - "pendukungdan" → "pendukung dan"
   - "Jatinegaradalam" → "Jatinegara dalam"
   - "prasaranadan" → "prasarana dan"
   - "volumedengan" → "volume dengan"
   - "tahunanuntuk" → "tahunan untuk"
   - "KesehatanMasyarakat" → "Kesehatan Masyarakat"
2. Perbaiki ejaan yang salah:
   - "Puskemas" → "Puskesmas"
   - "bedasarkan" → "berdasarkan"
   - "Managemen" → "Manajemen"
   - "layaanan" → "layanan"
   - "deksripsi" → "deskripsi"
3. Perbaiki spasi setelah tanda baca:
   - ",yang" → ", yang"
   - "sarana,yang" → "sarana, yang"
   - "kebijakan,peraturan" → "kebijakan, peraturan"
4. Perbaiki spasi sebelum tanda baca:
   - "Puskesmas  Jatinegara" (spasi ganda) → "Puskesmas Jatinegara"
5. JANGAN ubah kata yang sudah benar ejaannya

KEMBALIKAN HANYA JSON dengan format ini:
{{
  "revisi": [
    {{ "id": angka_id, "text": "teks hasil revisi" }}
  ]
}}

HARUS kembalikan SEMUA ID paragraf yang diberikan, meskipun tidak ada perubahan. Jika paragraf tidak berubah, kembalikan dengan teks aslinya.

Input Paragraf:
{}"#,
        prompt_input
    );

    let is_minimax = model.to_lowercase().contains("minimax") || endpoint.to_lowercase().contains("minimax");

    let messages = if is_minimax {
        json!([
            {"role": "system", "name": "system", "content": "You are a helpful copy editor. You only output valid JSON."},
            {"role": "user", "name": "user", "content": prompt}
        ])
    } else {
        json!([
            {"role": "system", "content": "You are a helpful copy editor. You only output valid JSON."},
            {"role": "user", "content": prompt}
        ])
    };

    let body = json!({
        "model": model,
        "messages": messages,
        "temperature": 0.3,
        "stream": false
    });

    let mut retries = 0u64;
    let max_retries = 3;

    loop {
        match client.post(endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await 
        {
            Ok(res) => {
                let status = res.status();
                
                if status == 429 || status.is_server_error() {
                    retries += 1;
                    if retries > max_retries {
                        return Err(format!("Max retries exceeded (status {})", status));
                    }
                    let delay_secs = 2u64.pow(retries as u32).min(32); // Exponential backoff 2,4,8,16,32s
                    println!("Rate limit backoff: {}s (retry {}/{})", delay_secs, retries, max_retries);
                    tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
                    continue;
                }

                if !status.is_success() {
                    let error_text = res.text().await.unwrap_or_default();
                    return Err(format!("API error {}: {}", status, error_text));
                }

                let response_text = res.text().await.unwrap_or_default();

                let clean_json = response_text
                    .trim()
                    .trim_start_matches("```json")
                    .trim_start_matches("```")
                    .trim_end_matches("```")
                    .trim();

                let content_text = if let Ok(json_res) = serde_json::from_str::<serde_json::Value>(clean_json) {
                    if let Some(content) = json_res["choices"][0]["message"]["content"].as_str() {
                        content.trim()
                            .trim_start_matches("```json")
                            .trim_start_matches("```")
                            .trim_end_matches("```")
                            .trim()
                            .to_string()
                    } else {
                        clean_json.to_string()
                    }
                } else {
                    clean_json.to_string()
                };

                match serde_json::from_str::<serde_json::Value>(&content_text) {
                    Ok(json_res) => {
                        let revisi_arr = json_res["revisi"].as_array()
                            .or_else(|| json_res.as_array());

                        let mut batch_findings = Vec::new();

                        if let Some(arr) = revisi_arr {
                            for item in arr {
                                if let (Some(id_val), Some(text_val)) = (item["id"].as_u64(), item["text"].as_str()) {
                                    let id = id_val as usize;
                                    let fixed_text = text_val.trim().to_string();

                                    if let Some(orig_p) = batch.iter().find(|p| p.index == id) {
                                        // Normalize whitespace for comparison (trim and collapse spaces)
                                        let normalize_ws = |s: &str| -> String {
                                            s.split_whitespace().collect::<Vec<_>>().join(" ")
                                        };
                                        let norm_fixed = normalize_ws(&fixed_text);
                                        let norm_orig = normalize_ws(&orig_p.text);

                                        if norm_fixed != norm_orig && !fixed_text.is_empty() {
                                            batch_findings.push(TypoFinding {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                original: orig_p.text.clone(),
                                                suggestion: fixed_text,
                                                context: orig_p.text.clone(),
                                                severity: TypoSeverity::Info,
                                                position: TypoPosition {
                                                    paragraph: orig_p.index,
                                                    start: 0,
                                                    end: orig_p.text.len(),
                                                },
                                                source: TypoSource::Ai,
                                                status: TypoStatus::Accepted,
                                            });
                                        }
                                    }
                                }
                            }
                        }

                        return Ok(batch_findings);
                    }
                    Err(e) => {
                        return Err(format!("Failed to parse JSON response: {}", e));
                    }
                }
            }
            Err(e) => {
                retries += 1;
                if retries > max_retries {
                    return Err(format!("Connection error after {} retries: {}", max_retries, e));
                }
                let delay_secs = retries * 3;
                tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
            }
        }
    }
}

// ────────────────────────────────────────────────────────────────
// Build TypoFinding entries from auto-fix map
// ────────────────────────────────────────────────────────────────

fn build_auto_findings(
    paragraphs: &[DocumentParagraph],
    auto_fix_map: &HashMap<usize, String>,
    exclude_indices: &std::collections::HashSet<usize>,
) -> Vec<TypoFinding> {
    auto_fix_map.iter()
        .filter(|(idx, _)| !exclude_indices.contains(idx))
        .filter_map(|(idx, fixed)| {
            paragraphs.iter().find(|p| p.index == *idx).map(|orig| {
                TypoFinding {
                    id: uuid::Uuid::new_v4().to_string(),
                    original: orig.text.clone(),
                    suggestion: fixed.clone(),
                    context: orig.text.clone(),
                    severity: TypoSeverity::Info,
                    position: TypoPosition { paragraph: *idx, start: 0, end: orig.text.len() },
                    source: TypoSource::Rules,
                    status: TypoStatus::Accepted,
                }
            })
        })
        .collect()
}

// ════════════════════════════════════════════════════════════════
// MAIN COMMAND: fix_document_with_ai
// ════════════════════════════════════════════════════════════════

#[derive(serde::Serialize)]
pub struct AiFixResult {
    pub message: String,
    pub findings: Vec<TypoFinding>,
}

#[tauri::command]
pub async fn fix_document_with_ai(
    app: AppHandle,
    input_path: String,
    output_path: String,
    api_key: String,
    base_url: String,
    model: String,
    fix_mode: String, // "quick", "smart", "deep"
) -> Result<AiFixResult, String> {
    let in_path = PathBuf::from(&input_path);
    let out_path = PathBuf::from(&output_path);

    println!("\n══════════════════════════════════════");
    println!("  DocFixer — {} MODE", fix_mode.to_uppercase());
    println!("══════════════════════════════════════\n");

    emit_progress(&app, 0, 0, "extracting", 0, "Extracting paragraphs from document...");

    let extracted = DocxParser::extract_text(&in_path)?;

    // Filter meaningful paragraphs
    let paragraphs_to_process: Vec<_> = extracted.paragraphs.into_iter()
        .filter(|p| {
            let trimmed = p.text.trim();
            if trimmed.is_empty() { return false; }
            if trimmed.len() < 30 && trimmed == trimmed.to_uppercase() && !trimmed.contains(' ') {
                return false;
            }
            trimmed.len() >= 15
        })
        .collect();

    let total_p = paragraphs_to_process.len();
    println!("Extracted {} meaningful paragraphs", total_p);

    // ══════════════════════════════════════════
    // STAGE 1: Auto-fix (regex-based, FREE)
    // ══════════════════════════════════════════
    emit_progress(&app, 0, 0, "auto-fixing", 0, 
        &format!("Stage 1: Auto-fixing {} paragraphs (double spaces, trim)...", total_p));

    let auto_fix_map = auto_fix_paragraphs(&paragraphs_to_process);
    let auto_fix_count = auto_fix_map.len();

    println!("Stage 1 ✓ Auto-fixed {} paragraphs (double spaces, trim)", auto_fix_count);

    // ── QUICK MODE: Export with auto-fixes only ──
    if fix_mode == "quick" {
        let auto_findings = build_auto_findings(&paragraphs_to_process, &auto_fix_map, &std::collections::HashSet::new());
        
        emit_progress(&app, 1, 1, "exporting", auto_findings.len(), 
            &format!("Exporting {} auto-fixes...", auto_findings.len()));
        DocxParser::export_ai_document(&in_path, &out_path, &auto_findings)?;
        
        emit_progress(&app, 1, 1, "done", auto_findings.len(),
            &format!("Quick fix done! {} paragraphs auto-fixed (0 AI tokens used).", auto_findings.len()));
        
        println!("\n✅ Quick fix complete: {} auto-fixes, 0 AI tokens", auto_findings.len());
        return Ok(AiFixResult {
            message: format!("Quick fix complete. {} paragraphs auto-fixed.", auto_findings.len()),
            findings: auto_findings
        });
    }

    // ══════════════════════════════════════════
    // STAGE 2: Dictionary Pre-filter (smart mode only)
    // ══════════════════════════════════════════

    // Build pre-fixed paragraphs (apply auto-fixes before dictionary check)
    let pre_fixed_paragraphs: Vec<DocumentParagraph> = paragraphs_to_process.iter()
        .map(|p| {
            if let Some(fixed) = auto_fix_map.get(&p.index) {
                DocumentParagraph { index: p.index, text: fixed.clone() }
            } else {
                p.clone()
            }
        })
        .collect();

    let paragraphs_for_ai = if fix_mode == "smart" {
        emit_progress(&app, 0, 0, "pre-filtering", 0, 
            "Stage 2: Dictionary pre-filter — checking which paragraphs need AI...");

        let app_clone = app.clone();
        let check_paras = pre_fixed_paragraphs.clone();
        
        // Regex patterns to detect word blend issues that dictionary won't catch
fn has_blend_issues(text: &str) -> bool {
    // Comprehensive blend patterns from the analysis document
    // Use contains-style matching (no leading \b) for blends that may be in middle of words
    let blend_patterns = [
        // ====================
        // SPECIFIC KNOWN BLENDS (use word boundaries where possible)
        // ====================
        // These patterns look for the blend pattern anywhere in text
        r"yangakan",
        r"yangdan",
        r"yanguntuk",
        r"yangdengan",
        r"yangmemerlukan",
        r"dengandan",
        r"denganuntuk",
        r"untukdan",
        r"untukmenyelesaikan",
        r"volumedengan",
        r"tahunanuntuk",
        r"pendukungdan",
        r"kegiatankegiatanyangakan",
        r"kegiatanyangakan",
        r"kegiatanbaruyang",
        r"masyarakatakan",
        r"lintassektoral",
        r"analisiskesehatan",
        r"pengembangansecara",
        r"tahunsebelumnya",
        r"terangkumdalamusulan",
        r"pembiayaandandukungan",
        r"baiksecara",
        r"usulanpembiayaan",
        r"prasaranadan",
        r"telahditetapkan",
        r"protokolklinis",
        r"disampaikanke",
        r"maupunpenulisan",
        r"unitupaya",
        r"FKTPatau",
        r"UKMdengan",
        r"perubahandokumen",
        r"harusdapat",
        r"Penyimpanandokumen",
        r"Pendistribusiandokumen",
        r"PenyusunanPerubahanDokumen",
        r"PengendaliDokumen",
        r"penomoranDokumen",
        r"diberinomor",
        r"FKTPagarmembuatkebijakan",
        r"tatanaskah",
        r"cekulang",
        r"menjadidua",
        r"dijalankann",
        r"ylesuai",
        r"harusdilakukan",
        r"yangakandilakukan",
        r"tilikesuaidengan",
        r"perbaikan/revisiisi",
        // ====================
        // JATINEGARA BLENDS (with leading boundary to avoid false matches)
        // ====================
        r"\bJatinegara",  // Jatinegara at start of word
        r"\bJatinegaradalam",
        r"\bJatinegaraini",
        r"\bJatinegaraber",
        r"\bJatinegarauntuk",
        r"\bJatinegarayang",
        r"\bJatinegaradengan",
        r"\bJatinegaradan",
        r"\bJatinegaraharus",
        r"\bJatinegarabisa",
        r"\bJatinegarajuga",
        r"\bJatinegarater",
        r"\bJatinegaralebar",
        r"\bJatinegaradiharapkan",
        // ====================
        // CAPITALIZED BLENDS (PascalCase - full word boundary)
        // ====================
        r"\b[A-Z][a-z]+[A-Z][a-z]+\b",
        r"\bKesehatanMasyarakat\b",
        r"\bWaktuPelaksanaan\b",
        // ====================
        // ENGLISH BLENDS
        // ====================
        r"\bhealthanalysis\b",
        // ====================
        // HYPHENATED ISSUES
        // ====================
        r"flow-chart",
        r"flowchart",
    ];

    for pattern in blend_patterns.iter() {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(text) {
                return true;
            }
        }
    }

    // ====================
    // MISSING SPACE AFTER PUNCTUATION
    // ====================
    // Check for comma/period directly followed by lowercase without space
    let punct_re = Regex::new(r",[a-z]|\.[a-z]").unwrap();
    if punct_re.is_match(text) {
        return true;
    }

    false
}

        let flagged = tokio::task::spawn_blocking(move || -> Vec<DocumentParagraph> {
            let state = app_clone.state::<EngineState>();
            let checker_guard = state.checker.lock().unwrap();

            if let Some(checker) = checker_guard.as_ref() {
                check_paras.into_iter()
                    .filter(|p| {
                        // Send to AI if: has spelling issues OR has blend patterns
                        checker.has_spelling_issues(&p.text) || has_blend_issues(&p.text)
                    })
                    .collect()
            } else {
                println!("⚠ Checker not initialized — sending all paragraphs to AI");
                check_paras
            }
        }).await.map_err(|e| e.to_string())?;

        let flagged_count = flagged.len();
        let skipped_count = total_p - flagged_count;
        let token_savings = if total_p > 0 { (skipped_count as f64 / total_p as f64 * 100.0) as u32 } else { 0 };
        
        println!("Stage 2 ✓ {} paragraphs flagged, {} clean (skipped) — ~{}% token savings", 
            flagged_count, skipped_count, token_savings);

        emit_progress(&app, 0, 0, "pre-filtered", auto_fix_count,
            &format!("Stage 2: {} flagged for AI, {} clean (skipped) — ~{}% token savings", 
                flagged_count, skipped_count, token_savings));

        // Short pause so user can see the pre-filter stats
        tokio::time::sleep(std::time::Duration::from_millis(800)).await;

        flagged
    } else {
        // "deep" mode — send all paragraphs to AI (with auto-fixes applied)
        println!("Deep mode — sending all {} paragraphs to AI", total_p);
        pre_fixed_paragraphs
    };

    // If no paragraphs need AI, just export auto-fixes
    if paragraphs_for_ai.is_empty() {
        let auto_findings = build_auto_findings(&paragraphs_to_process, &auto_fix_map, &std::collections::HashSet::new());
        
        emit_progress(&app, 1, 1, "done", auto_findings.len(),
            &format!("All paragraphs clean! {} auto-fixes applied, 0 AI tokens used.", auto_findings.len()));
        DocxParser::export_ai_document(&in_path, &out_path, &auto_findings)?;
        
        println!("\n✅ No AI needed: {} auto-fixes, 0 tokens", auto_findings.len());
        return Ok(AiFixResult {
            message: format!("No paragraphs needed AI fixing. {} auto-fixes applied.", auto_findings.len()),
            findings: auto_findings
        });
    }

    // ══════════════════════════════════════════
    // STAGE 3: AI Processing (concurrent batches)
    // ══════════════════════════════════════════

    if api_key.is_empty() {
        return Err("API key is required for Smart Fix and Deep Fix modes.".to_string());
    }

    let mut endpoint = base_url.trim_end_matches('/').to_string();
    if !endpoint.ends_with("/chat/completions") 
        && !endpoint.ends_with("/chatcompletion_v2") 
    {
        endpoint.push_str("/chat/completions");
    }

    // Track which paragraph indices are being sent to AI
    let ai_paragraph_indices: std::collections::HashSet<usize> = 
        paragraphs_for_ai.iter().map(|p| p.index).collect();

    let batches = build_smart_batches(paragraphs_for_ai);
    let total_batches = batches.len();

    println!("\nStage 3: Sending {} batches to AI (model: {})", total_batches, model);

    emit_progress(&app, 0, total_batches, "processing", auto_fix_count,
        &format!("Stage 3: Processing {} AI batches...", total_batches));

    // Concurrent processing with early termination
    let concurrency = 1; // Reduced to avoid rate limits
    let ai_findings = Arc::new(Mutex::new(Vec::new()));
    let completed_count = Arc::new(Mutex::new(0usize));
    let consecutive_errors = Arc::new(Mutex::new(0usize));
    let last_error = Arc::new(Mutex::new(String::new()));
    let should_stop = Arc::new(Mutex::new(false));
    let max_consecutive_errors = 20; // Relaxed for partial success
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .unwrap_or_else(|_| Client::new());

    let batch_results: Vec<_> = stream::iter(batches.into_iter().enumerate())
        .map(|(batch_idx, batch)| {
            let client = client.clone();
            let endpoint = endpoint.clone();
            let api_key = api_key.clone();
            let model = model.clone();
            let app = app.clone();
            let ai_findings = Arc::clone(&ai_findings);
            let completed_count = Arc::clone(&completed_count);
            let consecutive_errors = Arc::clone(&consecutive_errors);
            let last_error = Arc::clone(&last_error);
            let should_stop = Arc::clone(&should_stop);

            async move {
                if *should_stop.lock().await {
                    return (batch_idx, false);
                }

                match process_batch(&client, &endpoint, &api_key, &model, &batch, batch_idx).await {
                    Ok(batch_findings) => {
                        let fix_count = batch_findings.len();
                        *consecutive_errors.lock().await = 0;
                        ai_findings.lock().await.extend(batch_findings);
                        
                        let completed = {
                            let mut c = completed_count.lock().await;
                            *c += 1;
                            *c
                        };
                        
                        let total_fixes = ai_findings.lock().await.len();
                        emit_progress(&app, completed, total_batches, "processing", total_fixes + auto_fix_count, 
                            &format!("AI batch {}/{} done (+{} fixes)", completed, total_batches, fix_count));
                        
                        println!("  ✓ Batch {}/{}: {} fixes", completed, total_batches, fix_count);
                        (batch_idx, true)
                    }
                    Err(err) => {
                        let errs = {
                            let mut ce = consecutive_errors.lock().await;
                            *ce += 1;
                            *last_error.lock().await = err.clone();
                            *ce
                        };
                        
                        let completed = {
                            let mut c = completed_count.lock().await;
                            *c += 1;
                            *c
                        };
                        
                        eprintln!("  ✗ Batch {} failed: {} (consecutive: {})", batch_idx + 1, err, errs);
                        
                        emit_progress(&app, completed, total_batches, "warning", 
                            ai_findings.lock().await.len() + auto_fix_count, 
                            &format!("Batch {}/{} failed ({}), continuing...", batch_idx + 1, total_batches, err));
                        
                        eprintln!("⚠️ Batch {}/{} failed but continuing (consec: {})", batch_idx + 1, total_batches, errs);
                        
                        if errs >= max_consecutive_errors {
                            eprintln!("⚠️ Hit max consecutive errors ({}), but continuing with partial results", errs);
                            // Don't stop - allow partial success
                        }
                        
                        (batch_idx, false)
                    }
                }
            }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;

    // Check early termination
    let stopped_early = *should_stop.lock().await;
    let err_msg = last_error.lock().await.clone();
    let final_ai_findings = ai_findings.lock().await;
    
    if stopped_early && final_ai_findings.is_empty() {
        return Err(format!(
            "AI processing stopped after {} consecutive errors. Last error: {}. Please check your API key and model name.",
            max_consecutive_errors, err_msg
        ));
    }

    // ══════════════════════════════════════════
    // MERGE: Auto-fix + AI findings
    // ══════════════════════════════════════════
    
    // Auto-fix findings for paragraphs NOT sent to AI
    let auto_only_findings = build_auto_findings(&paragraphs_to_process, &auto_fix_map, &ai_paragraph_indices);
    
    // Combine: auto-fix (for clean paragraphs) + AI (for flagged paragraphs)
    let mut all_findings: Vec<TypoFinding> = Vec::new();
    all_findings.extend(auto_only_findings);
    all_findings.extend(final_ai_findings.clone());

    let success_count = batch_results.iter().filter(|(_, ok)| *ok).count();
    let fail_count = batch_results.len() - success_count;
    let ai_fix_count = final_ai_findings.len();

    println!("\n══════════════════════════════════════");
    println!("  PROCESSING SUMMARY");
    println!("══════════════════════════════════════");
    println!("  Total paragraphs:     {}", total_p);
    println!("  Auto-fixed (regex):   {}", auto_fix_count);
    println!("  Sent to AI:           {}", ai_paragraph_indices.len());
    println!("  AI fixes:             {}", ai_fix_count);
    println!("  Batches OK/Failed:    {}/{}", success_count, fail_count);
    println!("  Total fixes applied:  {}", all_findings.len());
    println!("══════════════════════════════════════\n");

    emit_progress(&app, total_batches, total_batches, "exporting", all_findings.len(), 
        "Applying all fixes and exporting document...");

    DocxParser::export_ai_document(&in_path, &out_path, &all_findings)?;

    let summary = format!(
        "Done! {} total fixes ({} auto, {} AI). {} paragraphs skipped (clean).",
        all_findings.len(),
        auto_fix_count,
        ai_fix_count,
        total_p - ai_paragraph_indices.len()
    );

    emit_progress(&app, total_batches, total_batches, "done", all_findings.len(), &summary);

    if fail_count > 0 {
        Ok(AiFixResult {
            message: format!("{} ({} batches failed)", summary, fail_count),
            findings: all_findings
        })
    } else {
        Ok(AiFixResult {
            message: summary,
            findings: all_findings
        })
    }
}