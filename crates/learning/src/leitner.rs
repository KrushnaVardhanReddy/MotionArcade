//! Leitner 5-Box Spaced Repetition Engine and Adaptive Scheduler.
//!
//! Conforms strictly to CONTRACT-LEARNING-004:
//! - Zero panics
//! - Deterministic spaced repetition
//! - Accuracy over speed (no latency penalty)
//! - 5-box promotion and demotion rules

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use crate::pack::{ContentError, ContentPack, LessonItem};

/// The 5 Leitner spaced repetition buckets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum LeitnerBox {
    /// Learning / Missed: reviewed every session / step.
    Box1 = 1,
    /// Practicing: reviewed every 2 steps.
    Box2 = 2,
    /// Familiar: reviewed every 4 steps.
    Box3 = 3,
    /// Proficient: reviewed every 8 steps.
    Box4 = 4,
    /// Mastered: reviewed rarely (every 16 steps).
    Box5 = 5,
}

impl LeitnerBox {
    /// Returns 0-based index (0..4) for array indexing.
    pub fn index(self) -> usize {
        (self as u8 - 1) as usize
    }

    /// Converts 0-based index to `LeitnerBox`.
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(LeitnerBox::Box1),
            1 => Some(LeitnerBox::Box2),
            2 => Some(LeitnerBox::Box3),
            3 => Some(LeitnerBox::Box4),
            4 => Some(LeitnerBox::Box5),
            _ => None,
        }
    }

    /// Review cadence interval (in steps) for this box.
    pub fn review_interval(self) -> u64 {
        match self {
            LeitnerBox::Box1 => 1,
            LeitnerBox::Box2 => 2,
            LeitnerBox::Box3 => 4,
            LeitnerBox::Box4 => 8,
            LeitnerBox::Box5 => 16,
        }
    }
}

/// Tracks learner progress, attempt counts, and box status for a specific item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeitnerProgress {
    pub item_id: String,
    pub current_box: LeitnerBox,
    pub total_attempts: u32,
    pub correct_attempts: u32,
    pub consecutive_correct: u32,
}

impl LeitnerProgress {
    /// Creates a fresh progress record starting in `Box1`.
    pub fn new(item_id: impl Into<String>) -> Self {
        Self {
            item_id: item_id.into(),
            current_box: LeitnerBox::Box1,
            total_attempts: 0,
            correct_attempts: 0,
            consecutive_correct: 0,
        }
    }

    /// Records an attempt on this item:
    /// - 2 consecutive correct answers promote the item to the next box.
    /// - Any incorrect answer immediately demotes the item to Box 1.
    pub fn record_attempt(&mut self, is_correct: bool) {
        self.total_attempts += 1;
        if is_correct {
            self.correct_attempts += 1;
            self.consecutive_correct += 1;
            if self.consecutive_correct >= 2 {
                self.promote();
                self.consecutive_correct = 0;
            }
        } else {
            self.consecutive_correct = 0;
            self.demote(); // Moves back to Box 1 for reinforcement
        }
    }

    /// Promotes the item to the next bucket, capping at `Box5`.
    pub fn promote(&mut self) {
        self.current_box = match self.current_box {
            LeitnerBox::Box1 => LeitnerBox::Box2,
            LeitnerBox::Box2 => LeitnerBox::Box3,
            LeitnerBox::Box3 => LeitnerBox::Box4,
            LeitnerBox::Box4 | LeitnerBox::Box5 => LeitnerBox::Box5,
        };
    }

    /// Demotes the item back to `Box1`.
    pub fn demote(&mut self) {
        self.current_box = LeitnerBox::Box1;
    }

    /// Returns accuracy ratio in range [0.0, 1.0].
    pub fn accuracy(&self) -> f32 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.correct_attempts as f32 / self.total_attempts as f32
        }
    }

    /// Returns true if item has achieved `Box5` mastery.
    pub fn is_mastered(&self) -> bool {
        self.current_box == LeitnerBox::Box5
    }
}

/// Adaptive scheduler delivering spaced repetition prompts deterministically.
#[derive(Debug, Clone)]
pub struct LeitnerScheduler {
    items: Vec<LessonItem>,
    progress: HashMap<String, LeitnerProgress>,
    box_queues: [VecDeque<String>; 5],
    step_count: u64,
    last_item_id: Option<String>,
}

impl LeitnerScheduler {
    /// Initializes the scheduler with lesson items.
    /// All items are placed in Box 1 for initial learning.
    pub fn new(items: Vec<LessonItem>) -> Self {
        let mut progress = HashMap::with_capacity(items.len());
        let mut box1_queue = VecDeque::with_capacity(items.len());

        for item in &items {
            progress.insert(item.id.clone(), LeitnerProgress::new(item.id.clone()));
            box1_queue.push_back(item.id.clone());
        }

        Self {
            items,
            progress,
            box_queues: [
                box1_queue,
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
                VecDeque::new(),
            ],
            step_count: 0,
            last_item_id: None,
        }
    }

    /// Creates a scheduler directly from a validated `ContentPack`.
    pub fn from_pack(pack: &ContentPack) -> Self {
        Self::new(pack.items.clone())
    }

    /// Restores a scheduler with existing progress mappings.
    pub fn from_pack_and_progress(
        pack: &ContentPack,
        saved_progress: HashMap<String, LeitnerProgress>,
    ) -> Self {
        let mut scheduler = Self::new(pack.items.clone());
        for (item_id, prog) in saved_progress {
            let target_box = prog.current_box;
            if let Some(entry) = scheduler.progress.get_mut(&item_id) {
                *entry = prog;
                // Move in queues
                for queue in &mut scheduler.box_queues {
                    queue.retain(|id| id != &item_id);
                }
                scheduler.box_queues[target_box.index()].push_back(item_id);
            }
        }
        scheduler
    }

    /// Selects the next lesson item according to deterministic spaced repetition schedules.
    pub fn next_item(&mut self) -> Option<&LessonItem> {
        if self.items.is_empty() {
            return None;
        }

        let next_step = self.step_count + 1;
        let target_box_idx = self.select_scheduled_box(next_step);

        let chosen_id = {
            let queue = &mut self.box_queues[target_box_idx];
            if queue.is_empty() {
                return None;
            }

            // If front item is same as last served and we have other candidates, rotate once
            if queue.len() > 1 && self.last_item_id.as_deref() == queue.front().map(|s| s.as_str())
            {
                if let Some(front) = queue.pop_front() {
                    queue.push_back(front);
                }
            }

            // Rotate front to back so items in the same box round-robin
            let id = queue.pop_front()?;
            queue.push_back(id.clone());
            id
        };

        self.step_count = next_step;
        self.last_item_id = Some(chosen_id.clone());

        self.items.iter().find(|i| i.id == chosen_id)
    }

    /// Records the learner's result (correct or incorrect) on the given item.
    /// Updates the item's Leitner box and reallocates it to the appropriate queue.
    pub fn record_result(
        &mut self,
        item_id: &str,
        is_correct: bool,
    ) -> Result<LeitnerBox, ContentError> {
        let progress = self
            .progress
            .get_mut(item_id)
            .ok_or_else(|| ContentError::ItemNotFound(item_id.to_string()))?;

        let old_box = progress.current_box;
        progress.record_attempt(is_correct);
        let new_box = progress.current_box;

        if old_box != new_box {
            // Remove from old queue
            self.box_queues[old_box.index()].retain(|id| id != item_id);
            // Push to back of new queue
            self.box_queues[new_box.index()].push_back(item_id.to_string());
        }

        Ok(new_box)
    }

    /// Selects which box queue to pull from based on the step counter:
    /// Prioritizes scheduled review of higher boxes (5 down to 2),
    /// falling back to Box 1, or the lowest non-empty box.
    fn select_scheduled_box(&self, step: u64) -> usize {
        // Check higher boxes from Box 5 down to Box 2 for due review
        for box_idx in (1..5).rev() {
            let b = match LeitnerBox::from_index(box_idx) {
                Some(b) => b,
                None => continue,
            };
            if step.is_multiple_of(b.review_interval()) && !self.box_queues[box_idx].is_empty() {
                return box_idx;
            }
        }

        // Default to Box 1
        if !self.box_queues[0].is_empty() {
            return 0;
        }

        // Fallback: pick the lowest non-empty box
        for box_idx in 1..5 {
            if !self.box_queues[box_idx].is_empty() {
                return box_idx;
            }
        }

        0
    }

    /// Returns the progress record for a specific item.
    pub fn get_progress(&self, item_id: &str) -> Option<&LeitnerProgress> {
        self.progress.get(item_id)
    }

    /// Returns reference to all progress records.
    pub fn all_progress(&self) -> &HashMap<String, LeitnerProgress> {
        &self.progress
    }

    /// Returns item counts in each box: `[Box1, Box2, Box3, Box4, Box5]`.
    pub fn box_counts(&self) -> [usize; 5] {
        [
            self.box_queues[0].len(),
            self.box_queues[1].len(),
            self.box_queues[2].len(),
            self.box_queues[3].len(),
            self.box_queues[4].len(),
        ]
    }

    /// Percentage of items that have reached `Box5` mastery in range [0.0, 1.0].
    pub fn mastery_percentage(&self) -> f32 {
        if self.items.is_empty() {
            return 0.0;
        }
        self.box_queues[4].len() as f32 / self.items.len() as f32
    }

    /// Returns true if all items in this pack have reached Box 5.
    pub fn is_all_mastered(&self) -> bool {
        !self.items.is_empty() && self.box_queues[4].len() == self.items.len()
    }

    /// Total number of attempts recorded across all items.
    pub fn total_attempts(&self) -> u32 {
        self.progress.values().map(|p| p.total_attempts).sum()
    }

    /// Total number of correct attempts across all items.
    pub fn total_correct(&self) -> u32 {
        self.progress.values().map(|p| p.correct_attempts).sum()
    }

    /// Overall accuracy ratio across all attempts in range [0.0, 1.0].
    pub fn overall_accuracy(&self) -> f32 {
        let attempts = self.total_attempts();
        if attempts == 0 {
            0.0
        } else {
            self.total_correct() as f32 / attempts as f32
        }
    }

    /// Returns the list of underlying lesson items.
    pub fn items(&self) -> &[LessonItem] {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_item(id: &str) -> LessonItem {
        LessonItem {
            id: id.to_string(),
            prompt: format!("Prompt {}", id),
            prompt_audio: None,
            answer: "Ans".to_string(),
            distractors: vec!["D1".to_string(), "D2".to_string()],
            skill: "test".to_string(),
        }
    }

    #[test]
    fn test_leitner_box_properties() {
        assert_eq!(LeitnerBox::Box1.index(), 0);
        assert_eq!(LeitnerBox::Box2.index(), 1);
        assert_eq!(LeitnerBox::Box3.index(), 2);
        assert_eq!(LeitnerBox::Box4.index(), 3);
        assert_eq!(LeitnerBox::Box5.index(), 4);

        assert_eq!(LeitnerBox::from_index(0), Some(LeitnerBox::Box1));
        assert_eq!(LeitnerBox::from_index(4), Some(LeitnerBox::Box5));
        assert_eq!(LeitnerBox::from_index(5), None);

        assert_eq!(LeitnerBox::Box1.review_interval(), 1);
        assert_eq!(LeitnerBox::Box2.review_interval(), 2);
        assert_eq!(LeitnerBox::Box3.review_interval(), 4);
        assert_eq!(LeitnerBox::Box4.review_interval(), 8);
        assert_eq!(LeitnerBox::Box5.review_interval(), 16);
    }

    #[test]
    fn test_leitner_progress_lifecycle_promotion() {
        let mut progress = LeitnerProgress::new("item1");
        assert_eq!(progress.current_box, LeitnerBox::Box1);
        assert_eq!(progress.total_attempts, 0);
        assert_eq!(progress.correct_attempts, 0);
        assert_eq!(progress.consecutive_correct, 0);

        // 1st correct attempt: stays in Box 1, consecutive = 1
        progress.record_attempt(true);
        assert_eq!(progress.current_box, LeitnerBox::Box1);
        assert_eq!(progress.total_attempts, 1);
        assert_eq!(progress.correct_attempts, 1);
        assert_eq!(progress.consecutive_correct, 1);

        // 2nd correct attempt: promotes to Box 2, consecutive resets to 0
        progress.record_attempt(true);
        assert_eq!(progress.current_box, LeitnerBox::Box2);
        assert_eq!(progress.total_attempts, 2);
        assert_eq!(progress.correct_attempts, 2);
        assert_eq!(progress.consecutive_correct, 0);

        // Promote through remaining boxes
        progress.record_attempt(true); // Box 2, 1st
        progress.record_attempt(true); // Promoted to Box 3
        assert_eq!(progress.current_box, LeitnerBox::Box3);

        progress.record_attempt(true); // Box 3, 1st
        progress.record_attempt(true); // Promoted to Box 4
        assert_eq!(progress.current_box, LeitnerBox::Box4);

        progress.record_attempt(true); // Box 4, 1st
        progress.record_attempt(true); // Promoted to Box 5 (Mastered)
        assert_eq!(progress.current_box, LeitnerBox::Box5);
        assert!(progress.is_mastered());

        // Further correct attempts cap at Box 5
        progress.record_attempt(true);
        progress.record_attempt(true);
        assert_eq!(progress.current_box, LeitnerBox::Box5);
    }

    #[test]
    fn test_leitner_progress_demotion_to_box_1() {
        let mut progress = LeitnerProgress::new("item1");
        // Fast forward to Box 4
        progress.current_box = LeitnerBox::Box4;
        progress.consecutive_correct = 1;

        // An incorrect attempt resets consecutive and demotes back to Box 1
        progress.record_attempt(false);
        assert_eq!(progress.current_box, LeitnerBox::Box1);
        assert_eq!(progress.consecutive_correct, 0);
        assert_eq!(progress.total_attempts, 1);
        assert_eq!(progress.correct_attempts, 0);
        assert!(!progress.is_mastered());
    }

    #[test]
    fn test_leitner_progress_accuracy() {
        let mut progress = LeitnerProgress::new("item1");
        assert_eq!(progress.accuracy(), 0.0);

        progress.record_attempt(true);
        progress.record_attempt(false);
        assert_eq!(progress.accuracy(), 0.5);
    }

    #[test]
    fn test_scheduler_initialization_and_first_item() {
        let items = vec![make_test_item("q1"), make_test_item("q2")];
        let mut scheduler = LeitnerScheduler::new(items);

        assert_eq!(scheduler.box_counts(), [2, 0, 0, 0, 0]);
        assert_eq!(scheduler.mastery_percentage(), 0.0);
        assert!(!scheduler.is_all_mastered());

        let next = scheduler.next_item().unwrap();
        assert_eq!(next.id, "q1");
    }

    #[test]
    fn test_scheduler_recording_result_and_promotion() {
        let items = vec![make_test_item("q1"), make_test_item("q2")];
        let mut scheduler = LeitnerScheduler::new(items);

        // Record 1st correct answer for q1
        let box_res = scheduler.record_result("q1", true).unwrap();
        assert_eq!(box_res, LeitnerBox::Box1);
        assert_eq!(scheduler.box_counts(), [2, 0, 0, 0, 0]);

        // Record 2nd correct answer for q1 -> promoted to Box 2
        let box_res = scheduler.record_result("q1", true).unwrap();
        assert_eq!(box_res, LeitnerBox::Box2);
        assert_eq!(scheduler.box_counts(), [1, 1, 0, 0, 0]);

        // q1 is in Box 2, q2 is in Box 1
        let p_q1 = scheduler.get_progress("q1").unwrap();
        assert_eq!(p_q1.current_box, LeitnerBox::Box2);
    }

    #[test]
    fn test_scheduler_demotion_moves_between_queues() {
        let items = vec![make_test_item("q1")];
        let mut scheduler = LeitnerScheduler::new(items);

        // Promote to Box 2
        scheduler.record_result("q1", true).unwrap();
        scheduler.record_result("q1", true).unwrap();
        assert_eq!(scheduler.box_counts(), [0, 1, 0, 0, 0]);

        // Incorrect answer -> demotes to Box 1
        let box_res = scheduler.record_result("q1", false).unwrap();
        assert_eq!(box_res, LeitnerBox::Box1);
        assert_eq!(scheduler.box_counts(), [1, 0, 0, 0, 0]);
    }

    #[test]
    fn test_scheduler_mastery_progression() {
        let items = vec![make_test_item("q1")];
        let mut scheduler = LeitnerScheduler::new(items);

        // 8 correct answers (2 per promotion from Box 1 -> 2 -> 3 -> 4 -> 5)
        for _ in 0..8 {
            scheduler.record_result("q1", true).unwrap();
        }

        assert_eq!(scheduler.box_counts(), [0, 0, 0, 0, 1]);
        assert_eq!(scheduler.mastery_percentage(), 1.0);
        assert!(scheduler.is_all_mastered());
        assert_eq!(scheduler.total_attempts(), 8);
        assert_eq!(scheduler.total_correct(), 8);
        assert_eq!(scheduler.overall_accuracy(), 1.0);
    }

    #[test]
    fn test_scheduler_does_not_repeat_same_item_consecutively_if_alternatives_exist() {
        let items = vec![make_test_item("q1"), make_test_item("q2")];
        let mut scheduler = LeitnerScheduler::new(items);

        let first = scheduler.next_item().unwrap().id.clone();
        let second = scheduler.next_item().unwrap().id.clone();

        assert_ne!(first, second);
    }
}
