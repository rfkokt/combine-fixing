use std::collections::HashSet;
use std::fs;
use std::path::Path;
use spellbook::Dictionary;

pub struct SpellEngine {
    id_dict: Dictionary,
    en_dict: Option<Dictionary>,
    custom_words: HashSet<String>,
}

impl SpellEngine {
    pub fn new(app_data_dir: &Path) -> Result<Self, String> {
        let id_aff_path = "dictionaries/id_ID.aff";
        let id_dic_path = "dictionaries/id_ID.dic";
        
        let id_aff_content = fs::read_to_string(id_aff_path).map_err(|e| format!("Failed to read id_ID.aff: {}", e))?;
        let id_dic_content = fs::read_to_string(id_dic_path).map_err(|e| format!("Failed to read id_ID.dic: {}", e))?;
        
        let id_dict = Dictionary::new(&id_aff_content, &id_dic_content)
            .map_err(|e| format!("Failed to parse id_ID dict: {:?}", e))?;

        // Try load english (optional)
        let en_dict = if let (Ok(aff), Ok(dic)) = (
            fs::read_to_string("dictionaries/en_US.aff"),
            fs::read_to_string("dictionaries/en_US.dic")
        ) {
            Dictionary::new(&aff, &dic).ok()
        } else {
            None
        };

        // Load custom dict
        let mut custom_words = HashSet::new();
        let custom_dic_path = app_data_dir.join("custom.dic");
        if let Ok(content) = fs::read_to_string(&custom_dic_path) {
            for line in content.lines() {
                let word = line.trim();
                if !word.is_empty() {
                    custom_words.insert(word.to_string());
                }
            }
        }

        Ok(Self {
            id_dict,
            en_dict,
            custom_words,
        })
    }

    pub fn is_valid_word(&self, word: &str) -> bool {
        // Strip punctuation
        let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
        if clean_word.is_empty() || clean_word.chars().all(|c| c.is_numeric()) {
            return true;
        }

        if self.custom_words.contains(clean_word) {
            return true;
        }

        // Check ID dict
        if self.id_dict.check(clean_word) {
            return true;
        }

        // Check EN dict if available
        if let Some(en) = &self.en_dict {
            if en.check(clean_word) {
                return true;
            }
        }

        false
    }

    pub fn suggest(&self, word: &str) -> Vec<String> {
        let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
        let mut suggestions = Vec::new();
        self.id_dict.suggest(clean_word, &mut suggestions);
        
        if let Some(en) = &self.en_dict {
            let mut en_suggestions = Vec::new();
            en.suggest(clean_word, &mut en_suggestions);
            suggestions.extend(en_suggestions);
        }
        
        // Deduplicate
        let mut unique = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for s in suggestions {
            if seen.insert(s.clone()) {
                unique.push(s);
            }
        }
        
        unique
    }
}
