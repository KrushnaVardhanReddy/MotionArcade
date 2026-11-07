# Learning Engine & Content Packs

This document covers how educational content integrates into games via offline-first TOML packs and the adaptive skill engine.

---

## 1. Decoupled Learning Model

```
Content pack ──► Learning engine ──► Game ──► Player moves ──► Result
                   (next_item)       (shows     (body pose)     (correct?
                        ▲             prompt)                    how fast?)
                        └──────────── updates skill model ◄──────────┘
```

1. **Game Mechanics:** Physical movement rules (deflecting orbs, hopping platforms).
2. **Content Packs:** Question bank (math facts, letter phonics, sight words).
3. **Learning Engine:** Determines sequence, difficulty, and review intervals.

---

## 2. Content Creation: Teacher-Friendly Interfaces

> **Guiding Principle:** TOML is strictly an internal storage and wire format. Teachers and parents should **never** have to write code, balance quotes, or debug text syntax.

To make lesson authoring as intuitive as Kahoot or Google Forms, MotionArcade provides three frictionless creation interfaces:

```
                     ┌──────────────────────────────────┐
                     │         TEACHER INTERFACE        │
                     └────────────────┬─────────────────┘
                                      │
          ┌───────────────────────────┼───────────────────────────┐
          ▼                           ▼                           ▼
  1. In-App Visual Form       2. Google Sheets / CSV       3. AI Prompt-to-Pack
  ─────────────────────       ──────────────────────       ────────────────────
  Form inside Teacher Portal:  Teachers paste/upload        "Generate 15 questions
  Question, Correct Answer,    spreadsheets of vocab        on 2nd grade fractions"
  Distractors 1, 2, 3          or math facts with 1 click   -> auto-generates pack
          │                           │                           │
          └───────────────────────────┼───────────────────────────┘
                                      │
                                      ▼
                      [ Auto-Generated .toml File ]
                  (Saved silently to disk / USB / app)
```

### A. In-App Visual Lesson Builder (Kahoot-Style Form)
Inside the Teacher Portal:
1. Tap **`[ + Create New Lesson ]`**.
2. Select Subject (`Math`, `Phonics`, `Science`) and Target Age Band (`4–6`, `7–9`, `10–12`).
3. Add cards visually:
   - **Prompt / Question:** (e.g. `12 + 15`)
   - **Correct Answer:** (Green field: `27`)
   - **Distractors / Wrong Choices:** (Red fields: `25`, `28`, `30`)
4. Tap **`[ Save & Play ]`** — the engine generates and validates the `.toml` file behind the scenes.

### B. Google Sheets / CSV Import (Spreadsheet Workflow)
Teachers frequently have question lists, vocabulary, or spelling words already in spreadsheets:
- A simple **`[ Import CSV ]`** button accepts a standard 3-column table:
  ```csv
  Question, Correct, Distractors
  3 x 4, 12, 10; 15; 16
  H2O is commonly called, Water, Hydrogen; Salt; Oxygen
  ```
- Instantly imports, validates, and packages the content into an offline lesson.

### C. AI "Prompt-to-Pack" Generator
- A teacher can simply type: *"Generate 20 questions on 3rd-grade animal habitats with 3 distractors each."*
- Generates the complete pack in seconds; the teacher reviews/edits before saving.

---

## 3. Underlying Content Pack Specification (TOML)

For developers, automated scrapers, and open-source contributors, the engine parses and stores lessons as lightweight TOML:

```toml
# content/bundled/math/number_facts_k2.toml

[pack]
id = "math-number-facts-k2"
title = "Number Facts (K–2)"
subject = "math"
age_band = "5-8"
reading_required = false    # Falls back to audio/icons if true
offline = true

[[items]]
id = "3plus2"
prompt = "3 + 2"
prompt_audio = "audio/prompts/3plus2.ogg"
answer = "5"
distractors = ["4", "6", "7"]
skill = "addition-within-10"

[[items]]
id = "7plus1"
prompt = "7 + 1"
prompt_audio = "audio/prompts/7plus1.ogg"
answer = "8"
distractors = ["6", "9", "10"]
skill = "addition-within-10"
```

### Pack Distribution & Installation

- **Bundled:** Ships directly inside the binary/asset directory (`content/bundled/`).
- **Offline Download:** Cached permanently into `content/downloaded/`.
- **USB / Local Share:** Installed directly via CLI or UI file picker without internet access:
  ```bash
  motionarcade install ./my_lesson_pack.toml
  ```

---

## 4. The Adaptive Engine

- **Leitner Box System:** Spaced repetition tracking without complex, unpredictable ML dependencies.
- **Accuracy Over Speed:** Children move at varying speeds and cameras have variable latency. Accuracy dictates mastery, not millisecond response time.
- **Positive Reinforcement:** Incorrect responses gently reveal the right answer and reschedule the item; fail states do not terminate play sessions.

