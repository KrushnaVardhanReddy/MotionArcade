//! Educational game bridge connecting content packs and Leitner repetition to gameplay.
//!
//! Conforms to CONTRACT-BRIDGE-005:
//! - Zero panics (`.unwrap()` / `.expect()`) in production code.
//! - Decoupled port trait (`LearningSessionPort`).
//! - Deterministic choice distribution across 4 interaction quadrants.

use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::leitner::LeitnerScheduler;
use crate::pack::{ContentError, ContentPack};

/// A single educational question wave presented in the game.
#[derive(Debug, Clone, PartialEq)]
pub struct QuestionWave {
    pub item_id: String,
    pub prompt: String,
    pub correct_answer: String,
    /// Choices distributed across quadrants (up to 4 options: 1 correct + 3 distractors)
    pub choices: Vec<OrbChoice>,
}

/// An orb choice bound to a specific interaction quadrant.
#[derive(Debug, Clone, PartialEq)]
pub struct OrbChoice {
    pub text: String,
    pub is_correct: bool,
    pub target_quadrant: u8, // 0: TopLeft, 1: TopRight, 2: BottomLeft, 3: BottomRight
}

/// Inbound game port for driving spaced-repetition questions.
pub trait LearningSessionPort: Send + 'static {
    /// Returns the active question prompt to display at the top of the HUD.
    fn current_prompt(&self) -> Option<String>;

    /// Spawns the choices for the next wave, populating 1 correct orb and distractors.
    fn next_question_wave(&mut self) -> Option<QuestionWave>;

    /// Records the user's selected orb.
    /// Returns true if correct, false if incorrect.
    fn evaluate_choice(&mut self, item_id: &str, chosen_text: &str) -> bool;

    /// Returns summary statistics for display (e.g., "7/8 Correct").
    fn session_stats(&self) -> (u32, u32); // (correct, total)

    /// Returns the distribution of items across the 5 Leitner boxes if backed by Leitner engine.
    fn box_counts(&self) -> Option<[usize; 5]> {
        None
    }
}

impl<T: LearningSessionPort + ?Sized> LearningSessionPort for Arc<Mutex<T>> {
    fn current_prompt(&self) -> Option<String> {
        match self.lock() {
            Ok(guard) => guard.current_prompt(),
            Err(e) => {
                tracing::warn!("Failed to lock learning session mutex: {}", e);
                None
            }
        }
    }

    fn next_question_wave(&mut self) -> Option<QuestionWave> {
        match self.lock() {
            Ok(mut guard) => guard.next_question_wave(),
            Err(e) => {
                tracing::warn!("Failed to lock learning session mutex: {}", e);
                None
            }
        }
    }

    fn evaluate_choice(&mut self, item_id: &str, chosen_text: &str) -> bool {
        match self.lock() {
            Ok(mut guard) => guard.evaluate_choice(item_id, chosen_text),
            Err(e) => {
                tracing::warn!("Failed to lock learning session mutex: {}", e);
                false
            }
        }
    }

    fn session_stats(&self) -> (u32, u32) {
        match self.lock() {
            Ok(guard) => guard.session_stats(),
            Err(e) => {
                tracing::warn!("Failed to lock learning session mutex: {}", e);
                (0, 0)
            }
        }
    }

    fn box_counts(&self) -> Option<[usize; 5]> {
        match self.lock() {
            Ok(guard) => guard.box_counts(),
            Err(e) => {
                tracing::warn!("Failed to lock learning session mutex: {}", e);
                None
            }
        }
    }
}

/// Educational session manager coordinating Leitner scheduler queries and question waves.
#[derive(Debug, Clone)]
pub struct LeitnerLearningSession {
    scheduler: LeitnerScheduler,
    pack: ContentPack,
    current_wave: Option<QuestionWave>,
    rng_state: u64,
    correct_count: u32,
    total_count: u32,
}

impl LeitnerLearningSession {
    /// Creates a new learning session from a validated content pack.
    pub fn new(pack: ContentPack) -> Self {
        let scheduler = LeitnerScheduler::from_pack(&pack);
        Self {
            scheduler,
            pack,
            current_wave: None,
            rng_state: 0x853c_49e6_748f_ea9b,
            correct_count: 0,
            total_count: 0,
        }
    }

    /// Loads a content pack from a TOML file and creates a learning session.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ContentError> {
        let pack = ContentPack::from_file(path)?;
        Ok(Self::new(pack))
    }

    /// Creates a session with an existing scheduler and content pack.
    pub fn from_scheduler_and_pack(scheduler: LeitnerScheduler, pack: ContentPack) -> Self {
        Self {
            scheduler,
            pack,
            current_wave: None,
            rng_state: 0x853c_49e6_748f_ea9b,
            correct_count: 0,
            total_count: 0,
        }
    }

    /// Returns a reference to the underlying Leitner scheduler.
    pub fn scheduler(&self) -> &LeitnerScheduler {
        &self.scheduler
    }

    /// Returns a mutable reference to the underlying Leitner scheduler.
    pub fn scheduler_mut(&mut self) -> &mut LeitnerScheduler {
        &mut self.scheduler
    }

    /// Returns a reference to the content pack.
    pub fn pack(&self) -> &ContentPack {
        &self.pack
    }

    /// Returns the current active question wave, if one is active.
    pub fn current_wave(&self) -> Option<&QuestionWave> {
        self.current_wave.as_ref()
    }

    /// Sets the pseudo-random generator seed for deterministic choice distribution in tests.
    pub fn set_seed(&mut self, seed: u64) {
        self.rng_state = if seed == 0 {
            0x853c_49e6_748f_ea9b
        } else {
            seed
        };
    }

    fn next_rand_u64(&mut self) -> u64 {
        let mut x = self.rng_state;
        if x == 0 {
            x = 0x853c_49e6_748f_ea9b;
        }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        x
    }
}

impl LearningSessionPort for LeitnerLearningSession {
    fn current_prompt(&self) -> Option<String> {
        self.current_wave.as_ref().map(|w| w.prompt.clone())
    }

    fn next_question_wave(&mut self) -> Option<QuestionWave> {
        let item = self.scheduler.next_item()?.clone();

        // Format prompt nicely if standard arithmetic
        let prompt = if !item.prompt.contains('=')
            && !item.prompt.contains('?')
            && (item.prompt.contains('+') || item.prompt.contains('-'))
        {
            format!("{} = ?", item.prompt)
        } else {
            item.prompt.clone()
        };

        // Collect choices: 1 correct + up to 3 distractors
        let mut choices_raw: Vec<(String, bool)> = Vec::with_capacity(4);
        choices_raw.push((item.answer.clone(), true));
        for distractor in item.distractors.iter().take(3) {
            choices_raw.push((distractor.clone(), false));
        }

        // Shuffle quadrant assignment [0, 1, 2, 3] using Fisher-Yates
        let mut quadrants = [0u8, 1, 2, 3];
        for i in (1..quadrants.len()).rev() {
            let j = (self.next_rand_u64() as usize) % (i + 1);
            quadrants.swap(i, j);
        }

        let mut choices = Vec::with_capacity(choices_raw.len());
        for (idx, (text, is_correct)) in choices_raw.into_iter().enumerate() {
            let target_quadrant = if idx < quadrants.len() {
                quadrants[idx]
            } else {
                (idx % 4) as u8
            };
            choices.push(OrbChoice {
                text,
                is_correct,
                target_quadrant,
            });
        }

        // Sort choices by target_quadrant for deterministic quadrant iteration
        choices.sort_by_key(|c| c.target_quadrant);

        let wave = QuestionWave {
            item_id: item.id.clone(),
            prompt,
            correct_answer: item.answer.clone(),
            choices,
        };

        self.current_wave = Some(wave.clone());
        Some(wave)
    }

    fn evaluate_choice(&mut self, item_id: &str, chosen_text: &str) -> bool {
        let is_correct = if let Some(ref wave) = self.current_wave {
            if wave.item_id == item_id {
                wave.correct_answer.trim() == chosen_text.trim()
            } else {
                self.pack
                    .items
                    .iter()
                    .find(|i| i.id == item_id)
                    .is_some_and(|i| i.answer.trim() == chosen_text.trim())
            }
        } else {
            self.pack
                .items
                .iter()
                .find(|i| i.id == item_id)
                .is_some_and(|i| i.answer.trim() == chosen_text.trim())
        };

        self.total_count = self.total_count.saturating_add(1);
        if is_correct {
            self.correct_count = self.correct_count.saturating_add(1);
        }

        if let Err(e) = self.scheduler.record_result(item_id, is_correct) {
            tracing::warn!("Failed to record Leitner attempt for '{}': {}", item_id, e);
        }

        is_correct
    }

    fn session_stats(&self) -> (u32, u32) {
        (self.correct_count, self.total_count)
    }

    fn box_counts(&self) -> Option<[usize; 5]> {
        Some(self.scheduler.box_counts())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::LessonItem;

    fn make_test_item(id: &str, prompt: &str, ans: &str) -> LessonItem {
        LessonItem {
            id: id.to_string(),
            prompt: prompt.to_string(),
            prompt_audio: None,
            answer: ans.to_string(),
            distractors: vec!["D1".to_string(), "D2".to_string(), "D3".to_string()],
            skill: "test_addition".to_string(),
        }
    }

    fn make_test_pack() -> ContentPack {
        ContentPack {
            pack: crate::pack::ContentPackHeader {
                id: "test-pack".to_string(),
                title: "Test".to_string(),
                subject: "math".to_string(),
                age_band: "5-8".to_string(),
                reading_required: false,
                offline: true,
            },
            items: vec![
                make_test_item("item1", "1 + 1", "2"),
                make_test_item("item2", "2 + 2", "4"),
            ],
        }
    }

    #[test]
    fn test_question_wave_generation_and_quadrants() {
        let pack = make_test_pack();
        let mut session = LeitnerLearningSession::new(pack);

        let wave = session.next_question_wave().unwrap();
        assert_eq!(wave.item_id, "item1");
        assert_eq!(wave.prompt, "1 + 1 = ?");
        assert_eq!(wave.correct_answer, "2");
        assert_eq!(wave.choices.len(), 4);

        // Assert all choices have distinct quadrants in 0..=3
        let mut quadrants: Vec<u8> = wave.choices.iter().map(|c| c.target_quadrant).collect();
        quadrants.sort();
        assert_eq!(quadrants, vec![0, 1, 2, 3]);

        // Exactly 1 correct choice
        let correct_choices: Vec<_> = wave.choices.iter().filter(|c| c.is_correct).collect();
        assert_eq!(correct_choices.len(), 1);
        assert_eq!(correct_choices[0].text, "2");
    }

    #[test]
    fn test_evaluate_choice_correct_and_stats() {
        let pack = make_test_pack();
        let mut session = LeitnerLearningSession::new(pack);
        session.next_question_wave().unwrap();

        assert_eq!(session.session_stats(), (0, 0));

        let is_correct = session.evaluate_choice("item1", "2");
        assert!(is_correct);
        assert_eq!(session.session_stats(), (1, 1));

        let is_correct_wrong = session.evaluate_choice("item1", "99");
        assert!(!is_correct_wrong);
        assert_eq!(session.session_stats(), (1, 2));
    }

    #[test]
    fn test_arc_mutex_session_port() {
        let pack = make_test_pack();
        let session = Arc::new(Mutex::new(LeitnerLearningSession::new(pack)));
        let mut port_wrapper = Arc::clone(&session);

        let wave = port_wrapper.next_question_wave().unwrap();
        assert_eq!(wave.item_id, "item1");

        let prompt = port_wrapper.current_prompt();
        assert_eq!(prompt, Some("1 + 1 = ?".to_string()));

        let ok = port_wrapper.evaluate_choice("item1", "2");
        assert!(ok);
        assert_eq!(port_wrapper.session_stats(), (1, 1));
    }
}
