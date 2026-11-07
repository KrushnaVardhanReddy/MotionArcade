//! 3-column teacher spreadsheet CSV importer.
//!
//! Transforms standard classroom teacher CSV spreadsheets:
//! `Question, Correct, Distractors` (e.g. `3 x 4, 12, 10; 15; 16`)
//! into validated, offline TOML `ContentPack`s.

use std::collections::HashSet;
use std::path::Path;

use crate::pack::{ContentError, ContentPack, ContentPackHeader, LessonItem};

/// Parses a 3-column (or multi-column) CSV string into a validated `ContentPack`.
///
/// Supported layouts:
/// - Header row (optional): `Question, Correct, Distractors` (case-insensitive)
/// - Delimited distractors: `10; 15; 16` (semicolon), `10|15|16` (pipe), or `10, 15, 16`
/// - Multi-column distractors: Column 0 = Question, Column 1 = Correct, Columns 2..N = Distractors
pub fn parse_csv_to_pack(
    csv_content: &str,
    pack_id: &str,
    title: &str,
    subject: &str,
    age_band: &str,
) -> Result<ContentPack, ContentError> {
    if csv_content.trim().is_empty() {
        return Err(ContentError::InvalidCsv("CSV content is empty".to_string()));
    }

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(csv_content.as_bytes());

    let mut items: Vec<LessonItem> = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();

    for (row_idx, record_res) in reader.records().enumerate() {
        let record = record_res.map_err(ContentError::Csv)?;
        if record.is_empty() {
            continue;
        }

        // Check if row 0 is a header row
        if row_idx == 0 && is_header_row(&record) {
            continue;
        }

        if record.len() < 3 {
            return Err(ContentError::InvalidCsv(format!(
                "Row {} has {} column(s); minimum 3 required (Question, Correct, Distractors)",
                row_idx + 1,
                record.len()
            )));
        }

        let prompt = match record.get(0) {
            Some(p) if !p.trim().is_empty() => p.trim().to_string(),
            _ => {
                return Err(ContentError::InvalidCsv(format!(
                    "Row {} is missing Question / Prompt",
                    row_idx + 1
                )));
            }
        };

        let answer = match record.get(1) {
            Some(a) if !a.trim().is_empty() => a.trim().to_string(),
            _ => {
                return Err(ContentError::InvalidCsv(format!(
                    "Row {} is missing Correct Answer",
                    row_idx + 1
                )));
            }
        };

        // Extract distractors from column 2 onwards
        let mut distractors = Vec::new();
        for col_idx in 2..record.len() {
            if let Some(field) = record.get(col_idx) {
                let parts = split_distractor_field(field);
                for p in parts {
                    let trimmed = p.trim().to_string();
                    if !trimmed.is_empty() && !distractors.contains(&trimmed) {
                        distractors.push(trimmed);
                    }
                }
            }
        }

        if distractors.is_empty() {
            return Err(ContentError::MissingDistractors {
                item_id: format!("row_{}", row_idx + 1),
            });
        }

        let item_id = generate_unique_id(&prompt, row_idx, &mut seen_ids);

        items.push(LessonItem {
            id: item_id,
            prompt,
            prompt_audio: None,
            answer,
            distractors,
            skill: format!("{}-core", subject.trim().to_lowercase()),
        });
    }

    if items.is_empty() {
        return Err(ContentError::EmptyPack);
    }

    let pack = ContentPack {
        pack: ContentPackHeader {
            id: pack_id.trim().to_string(),
            title: title.trim().to_string(),
            subject: subject.trim().to_string(),
            age_band: age_band.trim().to_string(),
            reading_required: false,
            offline: true,
        },
        items,
    };

    // Validate overall pack integrity (e.g. distractors don't match answers)
    pack.validate()?;

    Ok(pack)
}

/// Imports a CSV spreadsheet from disk, parses it into a `ContentPack`, and writes it as TOML.
pub fn import_csv_file_to_toml_file(
    csv_path: impl AsRef<Path>,
    toml_path: impl AsRef<Path>,
    pack_id: &str,
    title: &str,
    subject: &str,
    age_band: &str,
) -> Result<ContentPack, ContentError> {
    let csv_content = std::fs::read_to_string(csv_path)?;
    let pack = parse_csv_to_pack(&csv_content, pack_id, title, subject, age_band)?;
    pack.save_to_file(toml_path)?;
    Ok(pack)
}

/// Detects if a record resembles a CSV header row (e.g. `Question, Correct, Distractors`).
fn is_header_row(record: &csv::StringRecord) -> bool {
    let col0 = record.get(0).unwrap_or("").trim().to_lowercase();
    let col1 = record.get(1).unwrap_or("").trim().to_lowercase();

    let matches_col0 = col0 == "question" || col0 == "prompt" || col0 == "item";
    let matches_col1 = col1 == "correct" || col1 == "answer" || col1 == "correct answer";

    matches_col0 && matches_col1
}

/// Splits a distractor field by `;`, `|`, or `,` depending on what delimiter is present.
fn split_distractor_field(field: &str) -> Vec<&str> {
    if field.contains(';') {
        field.split(';').collect()
    } else if field.contains('|') {
        field.split('|').collect()
    } else if field.contains(',') {
        field.split(',').collect()
    } else {
        vec![field]
    }
}

/// Generates a clean, human-readable unique item ID based on the prompt.
fn generate_unique_id(prompt: &str, row_idx: usize, seen: &mut HashSet<String>) -> String {
    let mut base: String = prompt
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();

    // Collapse multiple consecutive underscores
    let mut clean_base = String::with_capacity(base.len());
    let mut last_was_underscore = false;
    for c in base.drain(..) {
        if c == '_' {
            if !last_was_underscore && !clean_base.is_empty() {
                clean_base.push('_');
                last_was_underscore = true;
            }
        } else {
            clean_base.push(c);
            last_was_underscore = false;
        }
    }

    let clean_base = clean_base.trim_matches('_');
    let candidate = if clean_base.is_empty() {
        format!("item_{:02}", row_idx + 1)
    } else if clean_base.len() > 24 {
        clean_base[..24].trim_matches('_').to_string()
    } else {
        clean_base.to_string()
    };

    let mut final_id = candidate.clone();
    let mut counter = 1;
    while seen.contains(&final_id) {
        final_id = format!("{}_{}", candidate, counter);
        counter += 1;
    }

    seen.insert(final_id.clone());
    final_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_3_column_csv() {
        let csv_data = r#"Question, Correct, Distractors
3 x 4, 12, 10; 15; 16
H2O is commonly called, Water, Hydrogen; Salt; Oxygen
"#;
        let pack =
            parse_csv_to_pack(csv_data, "quiz-1", "Classroom Quiz", "Science", "7-9").unwrap();

        assert_eq!(pack.pack.id, "quiz-1");
        assert_eq!(pack.pack.title, "Classroom Quiz");
        assert_eq!(pack.pack.subject, "Science");
        assert_eq!(pack.pack.age_band, "7-9");
        assert_eq!(pack.items.len(), 2);

        let item1 = &pack.items[0];
        assert_eq!(item1.prompt, "3 x 4");
        assert_eq!(item1.answer, "12");
        assert_eq!(item1.distractors, vec!["10", "15", "16"]);
        assert_eq!(item1.skill, "science-core");

        let item2 = &pack.items[1];
        assert_eq!(item2.prompt, "H2O is commonly called");
        assert_eq!(item2.answer, "Water");
        assert_eq!(item2.distractors, vec!["Hydrogen", "Salt", "Oxygen"]);
    }

    #[test]
    fn test_parse_pipe_delimited_distractors() {
        let csv_data = "Question, Correct, Distractors\n5 + 5, 10, 8|9|11|12\n";
        let pack = parse_csv_to_pack(csv_data, "math-5", "Math 5", "Math", "5-8").unwrap();
        assert_eq!(pack.items.len(), 1);
        assert_eq!(pack.items[0].distractors, vec!["8", "9", "11", "12"]);
    }

    #[test]
    fn test_parse_multi_column_distractors() {
        let csv_data = "Question,Correct,Distractor 1,Distractor 2,Distractor 3\n7 + 3,10,8,9,11\n";
        let pack = parse_csv_to_pack(csv_data, "math-add", "Addition", "Math", "5-8").unwrap();
        assert_eq!(pack.items.len(), 1);
        assert_eq!(pack.items[0].distractors, vec!["8", "9", "11"]);
    }

    #[test]
    fn test_csv_importer_handles_whitespace_and_header_variants() {
        let csv_data = "  prompt , answer , distractors  \n  4 + 4  ,  8  ,  6 ; 7 ; 9  \n";
        let pack = parse_csv_to_pack(csv_data, "pack-ws", "Whitespace", "Math", "5-8").unwrap();
        assert_eq!(pack.items.len(), 1);
        assert_eq!(pack.items[0].prompt, "4 + 4");
        assert_eq!(pack.items[0].answer, "8");
        assert_eq!(pack.items[0].distractors, vec!["6", "7", "9"]);
    }

    #[test]
    fn test_csv_missing_columns_error() {
        let csv_data = "Question, Correct\n2 + 2, 4\n";
        let err = parse_csv_to_pack(csv_data, "bad", "Bad", "Math", "5-8").unwrap_err();
        match err {
            ContentError::InvalidCsv(msg) => assert!(msg.contains("minimum 3 required")),
            other => panic!("Expected InvalidCsv, got {:?}", other),
        }
    }

    #[test]
    fn test_csv_distractor_equals_answer_fails_validation() {
        let csv_data = "Question, Correct, Distractors\n2 + 2, 4, 3; 4; 5\n";
        let err = parse_csv_to_pack(csv_data, "bad", "Bad", "Math", "5-8").unwrap_err();
        match err {
            ContentError::InvalidDistractor { distractor, .. } => assert_eq!(distractor, "4"),
            other => panic!("Expected InvalidDistractor, got {:?}", other),
        }
    }

    #[test]
    fn test_import_csv_file_to_toml_file() {
        let csv_data = "Question, Correct, Distractors\n1 + 2, 3, 2; 4; 5\n";
        let dir = std::env::temp_dir();
        let csv_path = dir.join("test_import_input.csv");
        let toml_path = dir.join("test_import_output.toml");

        std::fs::write(&csv_path, csv_data).unwrap();

        let pack = import_csv_file_to_toml_file(
            &csv_path,
            &toml_path,
            "imported-pack",
            "Imported Pack",
            "Math",
            "5-8",
        )
        .unwrap();

        assert_eq!(pack.pack.id, "imported-pack");
        assert!(toml_path.exists());

        // Validate that saved TOML can be loaded back cleanly
        let loaded_pack = ContentPack::from_file(&toml_path).unwrap();
        assert_eq!(pack, loaded_pack);

        // Clean up temp files
        let _ = std::fs::remove_file(&csv_path);
        let _ = std::fs::remove_file(&toml_path);
    }
}
