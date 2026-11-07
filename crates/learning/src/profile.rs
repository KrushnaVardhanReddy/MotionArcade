//! Local profile persistence and teacher classroom session reports.
//!
//! Conforms to Task 4.4:
//! - Store progress JSON locally per student nickname / avatar.
//! - Generate clean CSV session summaries for classroom teachers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::leitner::{LeitnerProgress, LeitnerScheduler};
use crate::pack::ContentError;

/// Local profile storing student nickname, chosen avatar, and item mastery progress.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StudentProfile {
    pub nickname: String,
    pub avatar: String,
    pub progress: HashMap<String, LeitnerProgress>,
}

impl StudentProfile {
    /// Creates a new profile with the given nickname and avatar.
    pub fn new(nickname: impl Into<String>, avatar: impl Into<String>) -> Self {
        Self {
            nickname: nickname.into(),
            avatar: avatar.into(),
            progress: HashMap::new(),
        }
    }

    /// Saves the profile locally as JSON.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), ContentError> {
        let json = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.as_ref().parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Loads a student profile from a local JSON file.
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, ContentError> {
        let content = std::fs::read_to_string(path)?;
        let profile = serde_json::from_str(&content)?;
        Ok(profile)
    }

    /// Synchronizes progress from an active `LeitnerScheduler`.
    pub fn sync_from_scheduler(&mut self, scheduler: &LeitnerScheduler) {
        for (item_id, p) in scheduler.all_progress() {
            self.progress.insert(item_id.clone(), p.clone());
        }
    }
}

/// An individual item's row for classroom session summaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemSummaryRow {
    pub item_id: String,
    pub current_box: u8,
    pub attempts: u32,
    pub correct: u32,
    pub accuracy_pct: u32,
}

/// Teacher session report providing high-level and per-item performance metrics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionReport {
    pub student_nickname: String,
    pub pack_id: String,
    pub timestamp_epoch_secs: u64,
    pub total_attempts: u32,
    pub correct_attempts: u32,
    pub mastered_count: usize,
    pub total_items: usize,
    pub items: Vec<ItemSummaryRow>,
}

impl SessionReport {
    /// Generates a session report from a completed or in-progress `LeitnerScheduler`.
    pub fn from_scheduler(
        student_nickname: impl Into<String>,
        pack_id: impl Into<String>,
        timestamp: u64,
        scheduler: &LeitnerScheduler,
    ) -> Self {
        let mut items = Vec::with_capacity(scheduler.items().len());

        for item in scheduler.items() {
            if let Some(p) = scheduler.get_progress(&item.id) {
                let acc = if p.total_attempts == 0 {
                    0
                } else {
                    ((p.correct_attempts as f64 / p.total_attempts as f64) * 100.0).round() as u32
                };

                items.push(ItemSummaryRow {
                    item_id: item.id.clone(),
                    current_box: p.current_box as u8,
                    attempts: p.total_attempts,
                    correct: p.correct_attempts,
                    accuracy_pct: acc,
                });
            }
        }

        Self {
            student_nickname: student_nickname.into(),
            pack_id: pack_id.into(),
            timestamp_epoch_secs: timestamp,
            total_attempts: scheduler.total_attempts(),
            correct_attempts: scheduler.total_correct(),
            mastered_count: scheduler.box_counts()[4],
            total_items: scheduler.items().len(),
            items,
        }
    }

    /// Generates a CSV formatted string suitable for teacher spreadsheets.
    pub fn to_csv(&self) -> String {
        let mut out = String::new();
        // Summary header
        out.push_str("# MotionArcade Classroom Session Summary\n");
        out.push_str(&format!(
            "# Student: {}, Pack: {}, Timestamp: {}\n",
            self.student_nickname, self.pack_id, self.timestamp_epoch_secs
        ));
        let overall_acc = if self.total_attempts == 0 {
            0
        } else {
            ((self.correct_attempts as f64 / self.total_attempts as f64) * 100.0).round() as u32
        };
        out.push_str(&format!(
            "# Total Attempts: {}, Correct: {}, Accuracy: {}%, Mastered: {}/{}\n\n",
            self.total_attempts,
            self.correct_attempts,
            overall_acc,
            self.mastered_count,
            self.total_items
        ));

        // Column table
        out.push_str("ItemID,Box,Attempts,Correct,AccuracyPct\n");
        for row in &self.items {
            out.push_str(&format!(
                "{},{},{},{},{}\n",
                row.item_id, row.current_box, row.attempts, row.correct, row.accuracy_pct
            ));
        }

        out
    }

    /// Writes the CSV report directly to a file on disk.
    pub fn save_csv_to_file(&self, path: impl AsRef<Path>) -> Result<(), ContentError> {
        let csv = self.to_csv();
        if let Some(parent) = path.as_ref().parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(path, csv)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::LessonItem;

    fn make_item(id: &str) -> LessonItem {
        LessonItem {
            id: id.to_string(),
            prompt: format!("Prompt {}", id),
            prompt_audio: None,
            answer: "Ans".to_string(),
            distractors: vec!["D1".to_string(), "D2".to_string()],
            skill: "math".to_string(),
        }
    }

    #[test]
    fn test_student_profile_save_and_load() {
        let mut profile = StudentProfile::new("Alex", "avatar_cat");
        assert_eq!(profile.nickname, "Alex");
        assert_eq!(profile.avatar, "avatar_cat");
        assert!(profile.progress.is_empty());

        let mut progress = LeitnerProgress::new("item1");
        progress.record_attempt(true);
        progress.record_attempt(true);
        profile.progress.insert("item1".to_string(), progress);

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_student_profile.json");
        profile.save_to_file(&path).unwrap();

        let loaded = StudentProfile::load_from_file(&path).unwrap();
        assert_eq!(profile, loaded);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_session_report_csv_generation() {
        let items = vec![make_item("q1"), make_item("q2")];
        let mut scheduler = LeitnerScheduler::new(items);

        // Record attempts
        scheduler.record_result("q1", true).unwrap();
        scheduler.record_result("q1", true).unwrap();
        scheduler.record_result("q2", false).unwrap();

        let report = SessionReport::from_scheduler("Sam", "math-k2", 1700000000, &scheduler);
        assert_eq!(report.student_nickname, "Sam");
        assert_eq!(report.total_attempts, 3);
        assert_eq!(report.correct_attempts, 2);

        let csv = report.to_csv();
        assert!(csv.contains("# Student: Sam, Pack: math-k2"));
        assert!(csv.contains("ItemID,Box,Attempts,Correct,AccuracyPct"));
        assert!(csv.contains("q1,2,2,2,100"));
        assert!(csv.contains("q2,1,1,0,0"));
    }
}
