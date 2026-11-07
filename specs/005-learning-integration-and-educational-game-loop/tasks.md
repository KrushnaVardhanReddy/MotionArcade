# Tasks: Spec 005 — Learning Engine Integration with Deflecto-Bot

**Spec Reference:** [`specs/005-learning-integration-and-educational-game-loop/spec.md`](spec.md)  
**Contract Reference:** [`contracts/contract_005_learning_game_bridge.md`](../../contracts/contract_005_learning_game_bridge.md)  
**Quality Standards:** Zero `.unwrap()`, positive reinforcement on errors, deterministic headless tests.

---

## Task Breakdown

- [ ] **Task 5.1: Game Learning Context Contract (`GameLearningPort`)**
  - Define `GameLearningPort` in `crates/core/src/learning.rs` or `crates/learning/src/bridge.rs`.
  - Provide method to query active question prompt and multiple-choice options.
  - Provide method to record child's deflection choice (`record_answer(choice: &str)`).

- [ ] **Task 5.2: Deflecto-Bot Educational Wave Generator**
  - Update `DeflectoBot` in `crates/games/deflecto_bot/src/lib.rs` to accept an optional `LeitnerScheduler`.
  - When a wave spawns, distribute the correct answer and distractors into randomized quadrants.
  - Store the question prompt (e.g. `4 + 3 = ?`) for HUD display.

- [ ] **Task 5.3: Answer Evaluation & Leitner Recording**
  - On quadrant strike: evaluate if the hit orb matches `item.answer`.
  - If correct: spawn victory particles, advance Leitner item, increment score.
  - If incorrect: dissolve orb with soft warning particle, record attempt, maintain prompt hint.

- [ ] **Task 5.4: App Runner CLI & Bundled Pack Hookup**
  - Update `crates/app` to load `content/bundled/math/number_facts_k2.toml` by default (or via `--pack <path>`).
  - Render active question prompt at the top center of the native HUD.

- [ ] **Task 5.5: End-to-End Simulation Test**
  - Add integration test simulating a player answering 5 math questions correctly and 1 incorrectly.
  - Verify Leitner box distribution and session report generation.
