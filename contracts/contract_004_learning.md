# Contract 004: TOML Content Packs, CSV Importer & Leitner Engine

**Contract ID:** `CONTRACT-LEARNING-004`  
**Crate:** `crates/learning`  
**Consumers:** `crates/app`, `crates/games/*`  
**Guarantees:** Zero `.unwrap()`, 100% offline parsing, deterministic spaced repetition.

---

## 1. Content Pack Types

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentPackHeader {
    pub id: String,
    pub title: String,
    pub subject: String,
    pub age_band: String, // "4-6", "7-9", "10-12"
    pub reading_required: bool,
    pub offline: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LessonItem {
    pub id: String,
    pub prompt: String,
    pub prompt_audio: Option<String>,
    pub answer: String,
    pub distractors: Vec<String>,
    pub skill: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentPack {
    pub pack: ContentPackHeader,
    pub items: Vec<LessonItem>,
}
```

---

## 2. Leitner Box Adaptive Engine

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeitnerBox {
    Box1 = 1, // Learning / Missed: seen every session
    Box2 = 2,
    Box3 = 3,
    Box4 = 4,
    Box5 = 5, // Mastered: reviewed rarely
}

pub struct LeitnerProgress {
    pub item_id: String,
    pub current_box: LeitnerBox,
    pub total_attempts: u32,
    pub correct_attempts: u32,
    pub consecutive_correct: u32,
}

impl LeitnerProgress {
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

    fn promote(&mut self) {
        self.current_box = match self.current_box {
            LeitnerBox::Box1 => LeitnerBox::Box2,
            LeitnerBox::Box2 => LeitnerBox::Box3,
            LeitnerBox::Box3 => LeitnerBox::Box4,
            LeitnerBox::Box4 | LeitnerBox::Box5 => LeitnerBox::Box5,
        };
    }

    fn demote(&mut self) {
        self.current_box = LeitnerBox::Box1;
    }
}
```

---

## 3. CSV Importer Contract

```rust
pub fn parse_csv_to_pack(
    csv_content: &str,
    pack_id: &str,
    title: &str,
    subject: &str,
    age_band: &str,
) -> Result<ContentPack, ContentError>;
```

