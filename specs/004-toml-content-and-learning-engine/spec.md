# Spec 004: TOML Content Packs, CSV Importer & Leitner Engine

## Status
Draft / In Design

---

## 1. Goal
Provide the decoupled educational engine: TOML parsing, CSV import, Leitner 5-box spaced repetition, and student session reports.

---

## 2. Components
1. **TOML Deserializer:** Validates question schema, age bands, distractors, and audio references.
2. **CSV Importer:** Converts standard 3-column teacher spreadsheets into valid lesson packs.
3. **Leitner Box Adaptive Engine:**
   - Box 1: New / struggling items (reviews frequently).
   - Box 5: Mastered items (reviews rarely).
   - Accuracy > Speed: No penalty for slow movement.

---

## 3. Acceptance Criteria
- [ ] Loads bundled packs from `content/bundled/`.
- [ ] `--validate-content` flag flags schema mismatches or missing distractors.
- [ ] Correct/incorrect attempts update Leitner boxes deterministically.
