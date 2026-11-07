# Spec 005: Learning Engine Integration with Deflecto-Bot Game Loop

## Status
Approved / Ready for Implementation

---

## 1. Context & Motivation
Currently, `deflecto_bot` spawns orbs with random/procedural numbers, while `crates/learning` possesses the validated TOML question bank and Leitner spaced repetition engine.
Spec 005 integrates them: the game queries the learning engine for the active lesson item, populates incoming flying orbs with the correct answer and distractors across quadrants, and records the child's deflection results back into the Leitner scheduler.

---

## 2. Educational Game Flow

```
     ┌─────────────────────────────────────────────────────────┐
     │              crates/learning::LeitnerScheduler          │
     │              Active Item: "4 + 3" (Answer: "7")         │
     │              Distractors: ["5", "6", "8"]               │
     └────────────────────────────┬────────────────────────────┘
                                  │
                                  ▼
     ┌─────────────────────────────────────────────────────────┐
     │         crates/games/deflecto_bot::DeflectoBot          │
     │                                                         │
     │   Top Prompt Display: "4 + 3 = ?"                       │
     │   Incoming Orbs Spawned:                                │
     │     • Quadrant::TopLeft     ──► "5" (Wrong)             │
     │     • Quadrant::TopRight    ──► "6" (Wrong)             │
     │     • Quadrant::BottomRight ──► "7" (Correct Target!)   │
     │     • Quadrant::BottomLeft  ──► "8" (Wrong)             │
     └────────────────────────────┬────────────────────────────┘
                                  │
                                  ▼
     Child physically reaches into Bottom-Right quadrant to strike "7":
       1. Impact particle fireworks explode (Golden Stars).
       2. Sound trigger: Positive success chime.
       3. Score increments (+100 Energy Points).
       4. Leitner Progress updated: 1 step toward Box promotion.
       5. DeflectoBot fetches next scheduled lesson item.
```

---

## 3. Zero-Penalty Handling on Incorrect Deflection
If a child mistakenly deflects an incorrect distractor orb (e.g. "5"):
1. The incorrect orb dissolves with a gentle wobble animation (no harsh buzzer or fail screen).
2. The prompt displays a hint: *"Oops! Look for 7!"*
3. The Leitner engine demotes the item to Box 1 for reinforcement.
4. The child can still hit the correct orb to complete the wave.

---

## 4. Acceptance Criteria
- [ ] `deflecto_bot` loads and plays items from any `ContentPack` (including bundled `number_facts_k2.toml`).
- [ ] Flying orbs display question answers and distractors cleanly.
- [ ] Correct deflections update the Leitner scheduler.
- [ ] Standalone test validates a full 10-question sequence playing through headless simulation.
- [ ] Exactly 0 occurrences of `.unwrap()` in production code.
