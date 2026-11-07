//! Content pack data structures, TOML deserializer, and schema validators.
//!
//! Guarantees: Zero panics, offline parsing, robust schema validation.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

/// Error types encountered while loading, validating, or serializing content packs.
#[derive(thiserror::Error, Debug)]
pub enum ContentError {
    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid CSV structure: {0}")]
    InvalidCsv(String),

    #[error("Content pack contains no lesson items")]
    EmptyPack,

    #[error("Invalid pack header field '{field}': {reason}")]
    InvalidHeader { field: &'static str, reason: String },

    #[error("Duplicate item ID '{0}' found in content pack")]
    DuplicateItemId(String),

    #[error("Item '{item_id}' has invalid prompt: {reason}")]
    InvalidPrompt { item_id: String, reason: String },

    #[error("Item '{item_id}' has invalid answer: {reason}")]
    InvalidAnswer { item_id: String, reason: String },

    #[error("Item '{item_id}' has missing distractors (minimum 1 required)")]
    MissingDistractors { item_id: String },

    #[error("Item '{item_id}' has invalid distractor '{distractor}': {reason}")]
    InvalidDistractor {
        item_id: String,
        distractor: String,
        reason: String,
    },

    #[error("Item '{item_id}' references missing audio asset: {path}")]
    MissingAudioFile { item_id: String, path: String },

    #[error("Item with ID '{0}' not found")]
    ItemNotFound(String),
}

/// Metadata header describing the content pack target audience, subject, and offline status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentPackHeader {
    pub id: String,
    pub title: String,
    pub subject: String,
    pub age_band: String, // e.g. "4-6", "5-8", "7-9", "10-12"
    pub reading_required: bool,
    pub offline: bool,
}

/// A single flashcard / prompt item inside a content pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LessonItem {
    pub id: String,
    pub prompt: String,
    pub prompt_audio: Option<String>,
    pub answer: String,
    pub distractors: Vec<String>,
    pub skill: String,
}

/// Top-level content pack containing a header table and a list of lesson items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentPack {
    pub pack: ContentPackHeader,
    #[serde(default)]
    pub items: Vec<LessonItem>,
}

impl ContentPack {
    /// Deserializes and validates a `ContentPack` from a TOML string.
    pub fn from_toml_str(toml_str: &str) -> Result<Self, ContentError> {
        let pack: Self = toml::from_str(toml_str)?;
        pack.validate()?;
        Ok(pack)
    }

    /// Reads, deserializes, and validates a `ContentPack` from a filesystem path.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ContentError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_toml_str(&content)
    }

    /// Serializes the content pack to a formatted TOML string.
    pub fn to_toml_string(&self) -> Result<String, ContentError> {
        self.validate()?;
        let s = toml::to_string_pretty(self)?;
        Ok(s)
    }

    /// Serializes and writes the content pack to the target file path.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), ContentError> {
        let toml_str = self.to_toml_string()?;
        if let Some(parent) = path.as_ref().parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(path, toml_str)?;
        Ok(())
    }

    /// Validates internal schema integrity:
    /// - Non-empty header fields
    /// - Non-empty items list
    /// - Unique item IDs
    /// - Non-empty prompts, answers, skills
    /// - Distractor presence (at least 1), non-empty, distinct from answer, and non-duplicate.
    pub fn validate(&self) -> Result<(), ContentError> {
        // 1. Validate Header
        if self.pack.id.trim().is_empty() {
            return Err(ContentError::InvalidHeader {
                field: "id",
                reason: "pack ID cannot be empty".to_string(),
            });
        }
        if self.pack.title.trim().is_empty() {
            return Err(ContentError::InvalidHeader {
                field: "title",
                reason: "pack title cannot be empty".to_string(),
            });
        }
        if self.pack.subject.trim().is_empty() {
            return Err(ContentError::InvalidHeader {
                field: "subject",
                reason: "pack subject cannot be empty".to_string(),
            });
        }
        if self.pack.age_band.trim().is_empty() {
            return Err(ContentError::InvalidHeader {
                field: "age_band",
                reason: "pack age_band cannot be empty".to_string(),
            });
        }

        // 2. Validate Items Existence
        if self.items.is_empty() {
            return Err(ContentError::EmptyPack);
        }

        // 3. Validate Each Item and Check for Duplicate IDs
        let mut seen_ids: HashSet<String> = HashSet::with_capacity(self.items.len());

        for item in &self.items {
            let item_id = item.id.trim();
            if item_id.is_empty() {
                return Err(ContentError::InvalidPrompt {
                    item_id: item.id.clone(),
                    reason: "item ID cannot be empty".to_string(),
                });
            }

            if !seen_ids.insert(item_id.to_string()) {
                return Err(ContentError::DuplicateItemId(item_id.to_string()));
            }

            if item.prompt.trim().is_empty() {
                return Err(ContentError::InvalidPrompt {
                    item_id: item.id.clone(),
                    reason: "prompt text cannot be empty".to_string(),
                });
            }

            let answer_trimmed = item.answer.trim();
            if answer_trimmed.is_empty() {
                return Err(ContentError::InvalidAnswer {
                    item_id: item.id.clone(),
                    reason: "correct answer cannot be empty".to_string(),
                });
            }

            if item.distractors.is_empty() {
                return Err(ContentError::MissingDistractors {
                    item_id: item.id.clone(),
                });
            }

            let mut seen_distractors: HashSet<String> =
                HashSet::with_capacity(item.distractors.len());

            for distractor in &item.distractors {
                let dist_trimmed = distractor.trim();
                if dist_trimmed.is_empty() {
                    return Err(ContentError::InvalidDistractor {
                        item_id: item.id.clone(),
                        distractor: distractor.clone(),
                        reason: "distractor choice cannot be empty".to_string(),
                    });
                }

                if dist_trimmed.eq_ignore_ascii_case(answer_trimmed) {
                    return Err(ContentError::InvalidDistractor {
                        item_id: item.id.clone(),
                        distractor: distractor.clone(),
                        reason: "distractor matches the correct answer".to_string(),
                    });
                }

                let key = dist_trimmed.to_lowercase();
                if !seen_distractors.insert(key) {
                    return Err(ContentError::InvalidDistractor {
                        item_id: item.id.clone(),
                        distractor: distractor.clone(),
                        reason: "duplicate distractor choice".to_string(),
                    });
                }
            }

            if item.skill.trim().is_empty() {
                return Err(ContentError::InvalidPrompt {
                    item_id: item.id.clone(),
                    reason: "skill tag cannot be empty".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Verifies that any audio files referenced in `prompt_audio` exist relative to `assets_root`.
    pub fn validate_assets(&self, assets_root: impl AsRef<Path>) -> Result<(), ContentError> {
        let root = assets_root.as_ref();
        for item in &self.items {
            if let Some(ref audio_rel) = item.prompt_audio {
                let trimmed = audio_rel.trim();
                if trimmed.is_empty() {
                    return Err(ContentError::MissingAudioFile {
                        item_id: item.id.clone(),
                        path: audio_rel.clone(),
                    });
                }
                let audio_path = root.join(trimmed);
                if !audio_path.exists() {
                    return Err(ContentError::MissingAudioFile {
                        item_id: item.id.clone(),
                        path: audio_rel.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TOML: &str = r#"
[pack]
id = "test-pack"
title = "Test Addition"
subject = "math"
age_band = "5-8"
reading_required = false
offline = true

[[items]]
id = "item1"
prompt = "1 + 1"
prompt_audio = "audio/1plus1.ogg"
answer = "2"
distractors = ["1", "3", "4"]
skill = "addition"

[[items]]
id = "item2"
prompt = "2 + 2"
answer = "4"
distractors = ["3", "5"]
skill = "addition"
"#;

    #[test]
    fn test_valid_pack_toml_roundtrip() {
        let pack = ContentPack::from_toml_str(SAMPLE_TOML).unwrap();
        assert_eq!(pack.pack.id, "test-pack");
        assert_eq!(pack.items.len(), 2);
        assert_eq!(pack.items[0].id, "item1");
        assert_eq!(pack.items[0].answer, "2");
        assert_eq!(pack.items[0].distractors, vec!["1", "3", "4"]);
        assert_eq!(
            pack.items[0].prompt_audio.as_deref(),
            Some("audio/1plus1.ogg")
        );
        assert_eq!(pack.items[1].prompt_audio, None);

        let serialized = pack.to_toml_string().unwrap();
        let parsed_again = ContentPack::from_toml_str(&serialized).unwrap();
        assert_eq!(pack, parsed_again);
    }

    #[test]
    fn test_bundled_pack_loads_and_validates() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../content/bundled/math/number_facts_k2.toml"
        );
        let pack = ContentPack::from_file(path).unwrap();
        assert_eq!(pack.pack.id, "math-number-facts-k2");
        assert_eq!(pack.pack.subject, "math");
        assert_eq!(pack.pack.age_band, "5-8");
        assert!(!pack.pack.reading_required);
        assert!(pack.pack.offline);
        assert_eq!(pack.items.len(), 10);

        for item in &pack.items {
            assert!(!item.id.is_empty());
            assert!(!item.prompt.is_empty());
            assert!(!item.answer.is_empty());
            assert!(item.distractors.len() >= 3);
            assert!(!item.distractors.contains(&item.answer));
        }
    }

    #[test]
    fn test_empty_pack_error() {
        let toml_str = r#"
[pack]
id = "empty-pack"
title = "Empty"
subject = "math"
age_band = "5-8"
reading_required = false
offline = true
items = []
"#;
        let err = ContentPack::from_toml_str(toml_str).unwrap_err();
        match err {
            ContentError::EmptyPack => {}
            other => panic!("Expected EmptyPack, got {:?}", other),
        }
    }

    #[test]
    fn test_duplicate_item_id_error() {
        let toml_str = r#"
[pack]
id = "dup-pack"
title = "Duplicates"
subject = "math"
age_band = "5-8"
reading_required = false
offline = true

[[items]]
id = "same_id"
prompt = "1 + 1"
answer = "2"
distractors = ["3"]
skill = "addition"

[[items]]
id = "same_id"
prompt = "2 + 2"
answer = "4"
distractors = ["5"]
skill = "addition"
"#;
        let err = ContentPack::from_toml_str(toml_str).unwrap_err();
        match err {
            ContentError::DuplicateItemId(id) => assert_eq!(id, "same_id"),
            other => panic!("Expected DuplicateItemId, got {:?}", other),
        }
    }

    #[test]
    fn test_empty_prompt_or_answer_error() {
        let toml_str = r#"
[pack]
id = "pack"
title = "Title"
subject = "math"
age_band = "5-8"
reading_required = false
offline = true

[[items]]
id = "item1"
prompt = "   "
answer = "2"
distractors = ["3"]
skill = "addition"
"#;
        let err = ContentPack::from_toml_str(toml_str).unwrap_err();
        match err {
            ContentError::InvalidPrompt { item_id, .. } => assert_eq!(item_id, "item1"),
            other => panic!("Expected InvalidPrompt, got {:?}", other),
        }
    }

    #[test]
    fn test_missing_distractors_error() {
        let toml_str = r#"
[pack]
id = "pack"
title = "Title"
subject = "math"
age_band = "5-8"
reading_required = false
offline = true

[[items]]
id = "item1"
prompt = "1 + 1"
answer = "2"
distractors = []
skill = "addition"
"#;
        let err = ContentPack::from_toml_str(toml_str).unwrap_err();
        match err {
            ContentError::MissingDistractors { item_id } => assert_eq!(item_id, "item1"),
            other => panic!("Expected MissingDistractors, got {:?}", other),
        }
    }

    #[test]
    fn test_distractor_matches_answer_error() {
        let toml_str = r#"
[pack]
id = "pack"
title = "Title"
subject = "math"
age_band = "5-8"
reading_required = false
offline = true

[[items]]
id = "item1"
prompt = "1 + 1"
answer = "2"
distractors = ["3", "2"]
skill = "addition"
"#;
        let err = ContentPack::from_toml_str(toml_str).unwrap_err();
        match err {
            ContentError::InvalidDistractor {
                item_id,
                distractor,
                ..
            } => {
                assert_eq!(item_id, "item1");
                assert_eq!(distractor, "2");
            }
            other => panic!("Expected InvalidDistractor, got {:?}", other),
        }
    }

    #[test]
    fn test_duplicate_distractor_error() {
        let toml_str = r#"
[pack]
id = "pack"
title = "Title"
subject = "math"
age_band = "5-8"
reading_required = false
offline = true

[[items]]
id = "item1"
prompt = "1 + 1"
answer = "2"
distractors = ["3", "3"]
skill = "addition"
"#;
        let err = ContentPack::from_toml_str(toml_str).unwrap_err();
        match err {
            ContentError::InvalidDistractor { item_id, .. } => assert_eq!(item_id, "item1"),
            other => panic!("Expected InvalidDistractor, got {:?}", other),
        }
    }

    #[test]
    fn test_invalid_header_error() {
        let toml_str = r#"
[pack]
id = ""
title = "Title"
subject = "math"
age_band = "5-8"
reading_required = false
offline = true

[[items]]
id = "item1"
prompt = "1 + 1"
answer = "2"
distractors = ["3"]
skill = "addition"
"#;
        let err = ContentPack::from_toml_str(toml_str).unwrap_err();
        match err {
            ContentError::InvalidHeader { field, .. } => assert_eq!(field, "id"),
            other => panic!("Expected InvalidHeader, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_assets_missing_file() {
        let toml_str = r#"
[pack]
id = "audio-pack"
title = "Audio Pack"
subject = "math"
age_band = "5-8"
reading_required = false
offline = true

[[items]]
id = "item1"
prompt = "1 + 1"
prompt_audio = "audio/non_existent.ogg"
answer = "2"
distractors = ["3"]
skill = "addition"
"#;
        let pack = ContentPack::from_toml_str(toml_str).unwrap();
        let temp_dir = std::env::temp_dir();
        let err = pack.validate_assets(&temp_dir).unwrap_err();
        match err {
            ContentError::MissingAudioFile { item_id, path } => {
                assert_eq!(item_id, "item1");
                assert_eq!(path, "audio/non_existent.ogg");
            }
            other => panic!("Expected MissingAudioFile, got {:?}", other),
        }
    }
}
