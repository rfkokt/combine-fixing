use crate::models::{TypoFinding, TypoSeverity, TypoSource, TypoStatus, TypoPosition, DocumentParagraph};
use super::dictionary::SpellEngine;
use regex::Regex;
use uuid::Uuid;

pub struct CheckerEngine {
    spell_engine: SpellEngine,
    // Pre-compiled regexes
    word_regex: Regex,
    double_space_regex: Regex,
}

impl CheckerEngine {
    pub fn new(spell_engine: SpellEngine) -> Self {
        Self {
            spell_engine,
            word_regex: Regex::new(r"[\w]+").unwrap(),
            double_space_regex: Regex::new(r"  +").unwrap(),
        }
    }

    pub fn scan_paragraph(&self, paragraph: &DocumentParagraph) -> Vec<TypoFinding> {
        let mut findings = Vec::new();
        let text = &paragraph.text;

        // 1. Check double spaces
        for mat in self.double_space_regex.find_iter(text) {
            findings.push(TypoFinding {
                id: Uuid::new_v4().to_string(),
                original: mat.as_str().to_string(),
                suggestion: " ".to_string(),
                context: self.get_context(text, mat.start(), mat.end()),
                severity: TypoSeverity::Warning,
                position: TypoPosition {
                    paragraph: paragraph.index,
                    start: mat.start(),
                    end: mat.end(),
                },
                source: TypoSource::Rules,
                status: TypoStatus::Pending,
            });
        }

        // 2. Check spelling per word
        for mat in self.word_regex.find_iter(text) {
            let word = mat.as_str();
            
            // Skip numbers and short words
            if word.chars().all(|c| c.is_numeric()) || word.len() <= 2 {
                continue;
            }

            if !self.spell_engine.is_valid_word(word) {
                let suggestions = self.spell_engine.suggest(word);
                let suggestion = suggestions.first().cloned().unwrap_or_else(|| String::new());

                findings.push(TypoFinding {
                    id: Uuid::new_v4().to_string(),
                    original: word.to_string(),
                    suggestion,
                    context: self.get_context(text, mat.start(), mat.end()),
                    severity: TypoSeverity::Error,
                    position: TypoPosition {
                        paragraph: paragraph.index,
                        start: mat.start(),
                        end: mat.end(),
                    },
                    source: TypoSource::Dictionary,
                    status: TypoStatus::Pending,
                });
            }
        }

        findings
    }

    fn get_context(&self, text: &str, start: usize, end: usize) -> String {
        // Get ~30 chars before and after for context
        let context_len = 30;
        let c_start = if start > context_len { start - context_len } else { 0 };
        let c_end = if end + context_len < text.len() { end + context_len } else { text.len() };
        
        // Find safe UTF-8 boundaries
        let mut s = c_start;
        while s > 0 && !text.is_char_boundary(s) { s -= 1; }
        
        let mut e = c_end;
        while e < text.len() && !text.is_char_boundary(e) { e += 1; }
        
        text[s..e].to_string()
    }
}
