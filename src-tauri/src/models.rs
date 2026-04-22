use serde::{Deserialize, Serialize};

/// Typo severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TypoSeverity {
    Error,
    Warning,
    Info,
}

/// Source of the typo detection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TypoSource {
    Dictionary,
    Rules,
    Ai,
}

/// Status of a typo finding
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TypoStatus {
    Pending,
    Accepted,
    Rejected,
    Ignored,
}

/// Position of the typo in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypoPosition {
    pub paragraph: usize,
    pub start: usize,
    pub end: usize,
}

/// A single detected typo finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypoFinding {
    pub id: String,
    pub original: String,
    pub suggestion: String,
    pub context: String,
    pub severity: TypoSeverity,
    pub position: TypoPosition,
    pub source: TypoSource,
    pub status: TypoStatus,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub word_count: Option<usize>,
    pub paragraph_count: Option<usize>,
}

/// Extracted paragraph from a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentParagraph {
    pub index: usize,
    pub text: String,
}

/// Result of extracting text from a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedDocument {
    pub info: DocumentInfo,
    pub paragraphs: Vec<DocumentParagraph>,
}

/// Result of scanning a document for typos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub document: DocumentInfo,
    pub findings: Vec<TypoFinding>,
    pub total_words: usize,
    pub scan_duration_ms: u64,
}
