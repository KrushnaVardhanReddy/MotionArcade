# Contract 005: Learning Engine & Game Loop Bridge

**Contract ID:** `CONTRACT-BRIDGE-005`  
**Crates:** `crates/learning` & `crates/games/deflecto_bot`  
**Guarantees:** Zero `.unwrap()`, decoupled dependency (Deflecto-Bot can still run in standalone arcade mode without questions).

---

## 1. Educational Question Wave Contract

```rust
use crate::pack::LessonItem;

#[derive(Debug, Clone, PartialEq)]
pub struct QuestionWave {
    pub item_id: String,
    pub prompt: String,
    pub correct_answer: String,
    /// Choices distributed across quadrants (up to 4 options: 1 correct + 3 distractors)
    pub choices: Vec<OrbChoice>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrbChoice {
    pub text: String,
    pub is_correct: bool,
    pub target_quadrant: u8, // 0: TopLeft, 1: TopRight, 2: BottomLeft, 3: BottomRight
}
```

---

## 2. In-Game Learning Bridge Trait

```rust
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
}
```

---

## 3. DeflectoBot Configuration

```rust
impl DeflectoBot {
    /// Creates a standard arcade bot (numbers are procedural, no learning pack).
    pub fn new_arcade() -> Self;
    
    /// Creates an educational bot wired directly to a learning session.
    pub fn new_learning(session: Box<dyn LearningSessionPort>) -> Self;
}
```
