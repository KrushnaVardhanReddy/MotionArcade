# Spec 011: Multi-Subject Bundled Content Packs

## Status
Approved / Ready for Parallel Implementation

---

## 1. Context & Motivation
To give kids and teachers multiple subjects on Day 1, Spec 011 adds three high-quality bundled TOML lesson packs:
1. `content/bundled/literacy/letter_sounds_k1.toml` (26 phonics items for pre-readers).
2. `content/bundled/math/multiplication_basics.toml` (2x, 5x, 10x tables for ages 7–10).
3. `content/bundled/science/animal_habitats.toml` (Ocean, Jungle, Desert animals for ages 5–8).

---

## 2. Zero-Conflict File Scope
- Operates strictly in:
  - `content/bundled/literacy/letter_sounds_k1.toml`
  - `content/bundled/math/multiplication_basics.toml`
  - `content/bundled/science/animal_habitats.toml`
  - `crates/learning/tests/bundled_packs_validation_test.rs`
- Zero edits to `crates/app/` (reserved for Agent 1).

---

## 3. Acceptance Criteria
- [ ] All three packs pass schema validation via `crates/learning::pack::ContentPack::validate()`.
- [ ] Every item has valid prompt, answer, $\ge 3$ distractors, and skill category.
- [ ] Integration test verifies all bundled packs deserialize cleanly.
