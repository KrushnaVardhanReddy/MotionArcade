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

## 2. Content Pack Specification (TOML)

Content packs require **zero Rust code** to create:

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

## 3. The Adaptive Engine

- **Leitner Box System:** Spaced repetition tracking without complex, unpredictable ML dependencies.
- **Accuracy Over Speed:** Children move at varying speeds and cameras have variable latency. Accuracy dictates mastery, not millisecond response time.
- **Positive Reinforcement:** Incorrect responses gently reveal the right answer and reschedule the item; fail states do not terminate play sessions.

