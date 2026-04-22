use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zip::{ZipArchive, ZipWriter};
use zip::write::SimpleFileOptions;
use regex::Regex;

use crate::models::{DocumentInfo, DocumentParagraph, ExtractedDocument, TypoFinding};

pub struct DocxParser;

impl DocxParser {
    pub fn extract_text(path: &Path) -> Result<ExtractedDocument, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let size = file.metadata().map(|m| m.len()).unwrap_or(0);
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        
        let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {}", e))?;
        
        // Find document.xml
        let mut document_xml = String::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            if file.name() == "word/document.xml" {
                file.read_to_string(&mut document_xml).map_err(|e| format!("Failed to read xml: {}", e))?;
                break;
            }
        }

        if document_xml.is_empty() {
            return Err("word/document.xml not found in docx".to_string());
        }

        // Parse paragraphs using roxmltree
        let doc = roxmltree::Document::parse(&document_xml)
            .map_err(|e| format!("Failed to parse xml: {}", e))?;
            
        let mut paragraphs = Vec::new();
        let mut p_index = 0;
        
        // Find all w:p elements
        for node in doc.descendants().filter(|n: &roxmltree::Node| n.has_tag_name("p")) {
            // Get text from w:t elements inside w:p
            let mut text = String::new();
            for text_node in node.descendants().filter(|n: &roxmltree::Node| n.has_tag_name("t")) {
                if let Some(t) = text_node.text() {
                    text.push_str(t);
                }
            }
            
            if !text.is_empty() {
                paragraphs.push(DocumentParagraph {
                    index: p_index,
                    text,
                });
                p_index += 1;
            }
        }

        let word_count = paragraphs.iter()
            .map(|p| p.text.split_whitespace().count())
            .sum();

        Ok(ExtractedDocument {
            info: DocumentInfo {
                path: path.to_string_lossy().to_string(),
                name,
                size,
                word_count: Some(word_count),
                paragraph_count: Some(paragraphs.len()),
            },
            paragraphs,
        })
    }

    pub fn export_fixed_document(input_path: &Path, output_path: &Path, findings: &[TypoFinding]) -> Result<(), String> {
        let in_file = File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
        let mut archive = ZipArchive::new(in_file).map_err(|e| format!("Failed to read input zip: {}", e))?;
        
        let out_file = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
        let mut zip_writer = ZipWriter::new(out_file);
        
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
            
        // Pre-compile regexes for each finding
        let mut replacements = Vec::new();
        for f in findings {
            let is_alphanumeric = f.original.chars().all(|c| c.is_alphanumeric());
            let pattern = if is_alphanumeric {
                format!(r"\b{}\b", regex::escape(&f.original))
            } else {
                regex::escape(&f.original)
            };
            
            if let Ok(re) = Regex::new(&pattern) {
                replacements.push((re, f.suggestion.clone()));
            }
        }

        let wt_regex = Regex::new(r"(?s)(<w:t[^>]*>)(.*?)(</w:t>)").unwrap();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| format!("Failed to read zip file index: {}", e))?;
            let file_name = file.name().to_string();
            
            zip_writer.start_file(&file_name, options).map_err(|e| format!("Failed to start zip file: {}", e))?;
            
            if file_name == "word/document.xml" {
                let mut xml = String::new();
                file.read_to_string(&mut xml).map_err(|e| format!("Failed to read xml: {}", e))?;
                
                let modified_xml = wt_regex.replace_all(&xml, |caps: &regex::Captures| {
                    let open_tag = &caps[1];
                    let mut text_content = caps[2].to_string();
                    let close_tag = &caps[3];
                    
                    for (re, suggestion) in &replacements {
                        text_content = re.replace_all(&text_content, suggestion).to_string();
                    }
                    
                    format!("{}{}{}", open_tag, text_content, close_tag)
                }).to_string();
                
                zip_writer.write_all(modified_xml.as_bytes()).map_err(|e| format!("Failed to write xml: {}", e))?;
            } else {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).map_err(|e| format!("Failed to read raw file: {}", e))?;
                zip_writer.write_all(&buffer).map_err(|e| format!("Failed to write raw file: {}", e))?;
            }
        }
        
        zip_writer.finish().map_err(|e| format!("Failed to finish zip writer: {}", e))?;
        
        Ok(())
    }

    pub fn export_ai_document(input_path: &Path, output_path: &Path, findings: &[TypoFinding]) -> Result<(), String> {
        let in_file = File::open(input_path).map_err(|e| format!("Failed to open input file: {}", e))?;
        let mut archive = ZipArchive::new(in_file).map_err(|e| format!("Failed to read input zip: {}", e))?;
        
        let out_file = File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
        let mut zip_writer = ZipWriter::new(out_file);
        
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
            
        // Map of paragraph index to AI suggestion
        let mut ai_replacements = std::collections::HashMap::new();
        for f in findings {
            ai_replacements.insert(f.position.paragraph, f.suggestion.clone());
        }

        // Regex to match <w:p> ... </w:p>
        let wp_regex = Regex::new(r"(?s)(<w:p(?:\s[^>]*)?>)(.*?)(</w:p>)").unwrap();
        // Regex to match <w:pPr> ... </w:pPr> to preserve paragraph styling
        let wppr_regex = Regex::new(r"(?s)(<w:pPr.*?</w:pPr>)").unwrap();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| format!("Failed to read zip file index: {}", e))?;
            let file_name = file.name().to_string();
            
            zip_writer.start_file(&file_name, options).map_err(|e| format!("Failed to start zip file: {}", e))?;
            
            if file_name == "word/document.xml" {
                let mut xml = String::new();
                file.read_to_string(&mut xml).map_err(|e| format!("Failed to read xml: {}", e))?;
                
                let mut p_index = 0;
                
                let modified_xml = wp_regex.replace_all(&xml, |caps: &regex::Captures| {
                    let open_tag = &caps[1];
                    let inner = &caps[2];
                    let close_tag = &caps[3];
                    
                    // Does this paragraph have text nodes?
                    let has_text = inner.contains("<w:t") && inner.contains("</w:t>");
                    
                    if has_text {
                        if let Some(ai_text) = ai_replacements.get(&p_index) {
                            // Escape special XML characters in AI text
                            let safe_text = ai_text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;");
                            
                            // Extract paragraph properties <w:pPr> to keep alignment/spacing
                            let p_pr = wppr_regex.find(inner).map(|m| m.as_str()).unwrap_or("");
                            
                            p_index += 1;
                            return format!("{}{}<w:r><w:t>{}</w:t></w:r>{}", open_tag, p_pr, safe_text, close_tag);
                        }
                        p_index += 1;
                    }
                    
                    // No text or no replacement -> keep original
                    format!("{}{}{}", open_tag, inner, close_tag)
                }).to_string();
                
                zip_writer.write_all(modified_xml.as_bytes()).map_err(|e| format!("Failed to write xml: {}", e))?;
            } else {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).map_err(|e| format!("Failed to read raw file: {}", e))?;
                zip_writer.write_all(&buffer).map_err(|e| format!("Failed to write raw file: {}", e))?;
            }
        }
        
        zip_writer.finish().map_err(|e| format!("Failed to finish zip writer: {}", e))?;
        
        Ok(())
    }
}
