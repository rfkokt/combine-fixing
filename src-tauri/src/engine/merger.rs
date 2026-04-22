use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::{ZipArchive, ZipWriter};
use zip::write::SimpleFileOptions;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SeparatorType {
    None,
    PageBreak,
    SectionBreak,
    DoublePageBreak,
}

impl Default for SeparatorType {
    fn default() -> Self {
        SeparatorType::PageBreak
    }
}

impl SeparatorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SeparatorType::None => "none",
            SeparatorType::PageBreak => "page_break",
            SeparatorType::SectionBreak => "section_break",
            SeparatorType::DoublePageBreak => "double_page_break",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "none" => SeparatorType::None,
            "section_break" => SeparatorType::SectionBreak,
            "double_page_break" => SeparatorType::DoublePageBreak,
            _ => SeparatorType::PageBreak,
        }
    }
}

pub struct DocumentMergeResult {
    pub output_path: PathBuf,
    pub total_paragraphs: usize,
    pub total_words: usize,
    pub documents_merged: usize,
}

pub struct Merger;

impl Merger {
    pub fn merge_documents(
        input_paths: &[String],
        output_path: &Path,
        separator: SeparatorType,
    ) -> Result<DocumentMergeResult, String> {
        if input_paths.is_empty() {
            return Err("No input documents provided".to_string());
        }

        if input_paths.len() == 1 {
            return Self::copy_single_document(&PathBuf::from(&input_paths[0]), output_path);
        }

        let mut all_paragraphs = Vec::<(usize, Vec<u8>)>::new();
        let mut total_words = 0;
        let mut total_paragraphs = 0;
        let mut first_doc_rels: Option<String> = None;
        let mut first_doc_styles: Option<String> = None;
        let mut first_doc_settings: Option<String> = None;

        for (doc_idx, path_str) in input_paths.iter().enumerate() {
            let path = PathBuf::from(path_str);
            let file = File::open(&path).map_err(|e| format!("Failed to open {}: {}", path_str, e))?;
            let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to read {}: {}", path_str, e))?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i).map_err(|e| format!("Failed to read file {}: {}", i, e))?;
                let file_name = file.name().to_string();

                if file_name == "word/document.xml" {
                    let mut xml = Vec::new();
                    file.read_to_end(&mut xml).map_err(|e| format!("Failed to read document.xml: {}", e))?;

                    let word_count = Self::count_words_from_xml(&xml);
                    let para_count = Self::count_paragraphs_from_xml(&xml);
                    total_words += word_count;
                    total_paragraphs += para_count;

                    all_paragraphs.push((doc_idx, xml));
                } else if file_name == "word/_rels/document.xml.rels" && doc_idx == 0 {
                    let mut content = String::new();
                    file.read_to_string(&mut content).map_err(|e| format!("Failed to read rels: {}", e))?;
                    first_doc_rels = Some(content);
                } else if file_name == "word/styles.xml" && doc_idx == 0 {
                    let mut content = String::new();
                    file.read_to_string(&mut content).map_err(|e| format!("Failed to read styles: {}", e))?;
                    first_doc_styles = Some(content);
                } else if file_name == "word/settings.xml" && doc_idx == 0 {
                    let mut content = String::new();
                    file.read_to_string(&mut content).map_err(|e| format!("Failed to read settings: {}", e))?;
                    first_doc_settings = Some(content);
                }
            }
        }

        if all_paragraphs.is_empty() {
            return Err("No valid documents found".to_string());
        }

        let merged_xml = Self::build_merged_xml(&all_paragraphs, separator)?;

        let out_file = File::create(output_path).map_err(|e| format!("Failed to create output: {}", e))?;
        let mut zip_writer = ZipWriter::new(out_file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        let temp_base = PathBuf::from(&input_paths[0]);
        let base_docx = File::open(&temp_base).map_err(|e| format!("Failed to open base docx: {}", e))?;
        let mut base_archive = ZipArchive::new(base_docx).map_err(|e| format!("Failed to read base archive: {}", e))?;

        for i in 0..base_archive.len() {
            let mut file = base_archive.by_index(i).map_err(|e| format!("Failed to read zip index: {}", e))?;
            let file_name = file.name().to_string();

            zip_writer.start_file(&file_name, options).map_err(|e| format!("Failed to start file: {}", e))?;

            if file_name == "word/document.xml" {
                zip_writer.write_all(&merged_xml).map_err(|e| format!("Failed to write merged xml: {}", e))?;
            } else if file_name == "word/_rels/document.xml.rels" {
                if let Some(rels) = first_doc_rels.take() {
                    zip_writer.write_all(rels.as_bytes()).map_err(|e| format!("Failed to write rels: {}", e))?;
                } else {
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer).map_err(|e| format!("Failed to read: {}", e))?;
                    zip_writer.write_all(&buffer).map_err(|e| format!("Failed to write: {}", e))?;
                }
            } else if file_name == "word/styles.xml" {
                if let Some(styles) = first_doc_styles.take() {
                    zip_writer.write_all(styles.as_bytes()).map_err(|e| format!("Failed to write styles: {}", e))?;
                } else {
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer).map_err(|e| format!("Failed to read: {}", e))?;
                    zip_writer.write_all(&buffer).map_err(|e| format!("Failed to write: {}", e))?;
                }
            } else if file_name == "word/settings.xml" {
                if let Some(settings) = first_doc_settings.take() {
                    zip_writer.write_all(settings.as_bytes()).map_err(|e| format!("Failed to write settings: {}", e))?;
                } else {
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer).map_err(|e| format!("Failed to read: {}", e))?;
                    zip_writer.write_all(&buffer).map_err(|e| format!("Failed to write: {}", e))?;
                }
            } else {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).map_err(|e| format!("Failed to read: {}", e))?;
                zip_writer.write_all(&buffer).map_err(|e| format!("Failed to write: {}", e))?;
            }
        }

        zip_writer.finish().map_err(|e| format!("Failed to finish zip: {}", e))?;

        Ok(DocumentMergeResult {
            output_path: output_path.to_path_buf(),
            total_paragraphs,
            total_words,
            documents_merged: input_paths.len(),
        })
    }

    fn copy_single_document(input: &Path, output: &Path) -> Result<DocumentMergeResult, String> {
        std::fs::copy(input, output).map_err(|e| format!("Failed to copy: {}", e))?;

        let file = File::open(input).map_err(|e| format!("Failed to open: {}", e))?;
        let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to read archive: {}", e))?;

        let mut total_words = 0;
        let mut total_paragraphs = 0;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| format!("Failed to read: {}", e))?;
            if file.name() == "word/document.xml" {
                let mut xml = Vec::new();
                file.read_to_end(&mut xml).map_err(|e| format!("Failed to read xml: {}", e))?;
                total_words = Self::count_words_from_xml(&xml);
                total_paragraphs = Self::count_paragraphs_from_xml(&xml);
                break;
            }
        }

        Ok(DocumentMergeResult {
            output_path: output.to_path_buf(),
            total_paragraphs,
            total_words,
            documents_merged: 1,
        })
    }

    fn build_merged_xml(docs: &[(usize, Vec<u8>)], separator: SeparatorType) -> Result<Vec<u8>, String> {
        let page_break_xml = b"<w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>";
        let section_break_xml = b"<w:p><w:pPr><w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/><w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr></w:pPr></w:p>";
        let double_page_break_xml = b"<w:p><w:r><w:br w:type=\"page\"/></w:r></w:p><w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>";

        let separator_xml: Option<&[u8]> = match separator {
            SeparatorType::None => None,
            SeparatorType::PageBreak => Some(page_break_xml),
            SeparatorType::SectionBreak => Some(section_break_xml),
            SeparatorType::DoublePageBreak => Some(double_page_break_xml),
        };

        let body_start = b"<w:body>";
        let body_end_tag = b"</w:body>";
        let doc_end_tag = b"</w:document>";

        let mut result = Vec::new();

        result.extend_from_slice(b"<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        result.extend_from_slice(b"<w:document xmlns:wpc=\"http://schemas.microsoft.com/office/word/2010/wordprocessingCanvas\" xmlns:mc=\"http://schemas.openxmlformats.org/markup-compatibility/2006\" xmlns:o=\"urn:schemas-microsoft-com:office:office\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\" xmlns:v=\"urn:schemas-microsoft-com:vml\" xmlns:wp14=\"http://schemas.microsoft.com/office/word/2010/wordprocessingDrawing\" xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" xmlns:w10=\"urn:schemas-microsoft-com:office:word\" xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" xmlns:w14=\"http://schemas.microsoft.com/office/word/2010/wordml\" xmlns:wpg=\"http://schemas.microsoft.com/office/word/2010/wordprocessingGroup\" xmlns:wpi=\"http://schemas.microsoft.com/office/word/2010/wordprocessingInk\" xmlns:wne=\"http://schemas.microsoft.com/office/word/2006/wordml\" xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\" mc=\"http://schemas.openxmlformats.org/markup-compatibility/2006\"Ignorable=\"w14 wp14\">");
        result.extend_from_slice(body_start);

        for (idx, (_, xml)) in docs.iter().enumerate() {
            let xml_str = String::from_utf8_lossy(xml);

            if let Some(body_start_pos) = xml_str.find("<w:body>") {
                if let Some(body_end_pos) = xml_str.find("</w:body>") {
                    let inner_start = body_start_pos + 8;
                    let inner_end = body_end_pos;
                    let inner = &xml_str[inner_start..inner_end];

                    let clean_inner = inner.trim();
                    if !clean_inner.is_empty() {
                        if !result.ends_with(body_start) && result.len() > 0 {
                        }
                        result.extend_from_slice(clean_inner.as_bytes());
                    }
                }
            }

            if idx < docs.len() - 1 {
                if let Some(sep) = separator_xml {
                    result.extend_from_slice(sep);
                }
            }
        }

        result.extend_from_slice(body_end_tag);
        result.extend_from_slice(doc_end_tag);

        Ok(result)
    }

    fn count_words_from_xml(xml: &[u8]) -> usize {
        let xml_str = String::from_utf8_lossy(xml);
        let mut count = 0;

        let text_regex = regex::Regex::new(r"<w:t[^>]*>([^<]*)</w:t>").unwrap();
        for cap in text_regex.captures_iter(&xml_str) {
            if let Some(words) = cap.get(1) {
                count += words.as_str().split_whitespace().count();
            }
        }

        count
    }

    fn count_paragraphs_from_xml(xml: &[u8]) -> usize {
        let xml_str = String::from_utf8_lossy(xml);
        let para_regex = regex::Regex::new(r"<w:p[>\s]").unwrap();
        para_regex.find_iter(&xml_str).count()
    }

    pub fn get_document_preview(path: &Path) -> Result<String, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open: {}", e))?;
        let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to read archive: {}", e))?;

        let mut document_xml = String::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| format!("Failed to read: {}", e))?;
            if file.name() == "word/document.xml" {
                file.read_to_string(&mut document_xml).map_err(|e| format!("Failed to read xml: {}", e))?;
                break;
            }
        }

        let doc = roxmltree::Document::parse(&document_xml)
            .map_err(|e| format!("Failed to parse: {}", e))?;

        let mut preview_lines = Vec::new();
        let mut para_count = 0;

        for node in doc.descendants().filter(|n: &roxmltree::Node| n.has_tag_name("p")) {
            if para_count >= 5 {
                preview_lines.push("...".to_string());
                break;
            }

            let mut text = String::new();
            for text_node in node.descendants().filter(|n: &roxmltree::Node| n.has_tag_name("t")) {
                if let Some(t) = text_node.text() {
                    text.push_str(t);
                }
            }

            let trimmed = text.trim();
            if !trimmed.is_empty() {
                let display = if trimmed.len() > 100 {
                    format!("{}...", &trimmed[..100])
                } else {
                    trimmed.to_string()
                };
                preview_lines.push(display);
                para_count += 1;
            }
        }

        Ok(preview_lines.join("\n"))
    }
}