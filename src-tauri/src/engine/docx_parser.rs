use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

use crate::models::{DocumentInfo, DocumentParagraph, ExtractedDocument};

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
}
