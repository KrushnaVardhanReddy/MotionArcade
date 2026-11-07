# Tasks: Spec 004 — TOML Content Packs, CSV Import & Leitner Engine

**Spec Reference:** [`specs/004-toml-content-and-learning-engine/spec.md`](spec.md)  
**Quality Standards:** Accuracy > speed, zero penalty for wrong answers, 100% offline schema validation.

---

## Task Breakdown

- [ ] **Task 4.1: TOML Pack Deserializer & Validator**
  - Implement pack loader in `crates/learning/src/loader.rs` using `serde`.
  - Validate age bands, duplicate IDs, missing distractors, and audio file existence.
  - Expose `--validate-content` CLI check.

- [ ] **Task 4.2: Teacher CSV Importer**
  - Implement 3-column CSV spreadsheet parser (`Question, Correct, Distractors`).
  - Convert parsed rows into validated TOML pack on disk.

- [ ] **Task 4.3: Leitner 5-Box Spaced Repetition Engine**
  - Implement Leitner bucket tracker in `crates/learning/src/leitner.rs`.
  - Box 1 (frequent review) to Box 5 (mastered).
  - Score attempts based strictly on correctness (no penalty for camera latency).

- [ ] **Task 4.4: Local Profile & Session Reports**
  - Store progress JSON locally per student nickname / avatar.
  - Generate CSV session summary for classroom teachers.

