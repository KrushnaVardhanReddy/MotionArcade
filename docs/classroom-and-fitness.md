# Classroom Deployment, Adult Fitness & Product Roadmap

Details on school classroom operations, expansion into adult fitness coaching, and project development phases.

---

## 1. Classroom Operations

Designed for typical school environments (Chromebooks, smart boards, projectors, limited bandwidth).

### Hardware & Network Requirements
- Runs via **WebAssembly (WASM)** directly in Chrome / Edge / ChromeOS.
- **Offline Capable:** Can run fully air-gapped from local caching or USB sticks.
- Accommodates 2–4 active students at the camera while the remainder of the room observes or participates in team scoring.

### Teacher Management & Whole-Class Engagement
- **The "Audience Participation" Rule:** While 2–4 students are actively tracked at the camera as "Pilots", the game prompts the remaining seated students to cast collective answers (*"If you think the answer is Even, hands on your head! If Odd, raise both hands!"*). This prevents idle disengagement and turns screen time into a full-room cooperative brain break.
- **Teacher PIN:** Secures roster configuration, data exports, and game catalogs.
- **Privacy Protections:** Student profiles use avatars and nicknames (no emails, PII, or student ID numbers).
- **Session Reports:** Summarizes missed questions, group accuracy trends, and CSV export for grade books.

---

## 2. Adult Fitness & Skill Coaching (Phase 3)

The underlying engine handles full-body keypoint tracking and timing loops, allowing straightforward reuse for adult workout content.

### "Family Challenge" Mode (Parent vs. Kid)
The transition between children's educational play and adult fitness should not be a separate app silo.
- **Asymmetric Co-op / Versus:** A parent and child can play together simultaneously.
- **Adaptive Lanes:** The 7-year-old receives addition/spelling orbs scaled to child arm reach, while the parent's lane fires rapid squat, lunge, or jumping targets.
- **High Retention:** Transforms daily learning into a fun, shared family physical workout.

### Workout Packs
Uses the same TOML structure as learning packs:

```toml
[pack]
id = "hiit-core"
title = "15-Minute Core & Cardio"
subject = "fitness"
age_band = "adult"
type = "workout"

[[exercises]]
id = "squat-set-1"
name = "Bodyweight Squats"
reps = 12
detection = "squat"
form_cue = "Lower hips past knees!"
form_threshold = 0.65
rest_after_seconds = 15
```

### Feature Differences from Kids Mode
- **Metric-Driven:** Rep counting, cadence, interval timers, and streak metrics.
- **Form Verification:** Coarse angle checks (hip-to-torso, knee tracking) with non-medical coaching guidance.
- **Extended Sessions:** 15–30 minute workout structures.

---

## 3. Product Roadmap

### Phase 0 — Pipeline Validation
- [ ] Camera capture (`nokhwa`) + MoveNet pose model (`ort`)
- [ ] 17-keypoint skeleton rendering and One Euro smoothing
- [ ] Deflecto-Bot basic play mechanics (procedural orbs)
- [ ] Recorded mock input playback (`--mock-input`)

### Phase 1 — Kids at Home (Primary Focus)
- [ ] Calibration flow (camera framing, latency sync)
- [ ] Bundled starter content packs (K–2 math and literacy)
- [ ] Leitner box adaptive learning engine
- [ ] Deflecto-Bot and River Hop learning modes
- [ ] Local profile storage, break prompts, parent summary
- [ ] Browser (WASM) build target

### Phase 2 — Classrooms
- [ ] Classroom roster, turn rotation, teacher PIN & dashboard
- [ ] Offline USB package installation
- [ ] Storm Surge and Body Spell games
- [ ] Third-party sandboxed plugin system (WASM `.mgpkg`)
- [ ] FERPA / COPPA compliance audit

### Phase 3 — Adult Fitness
- [ ] Workout TOML parser & rep counting mechanics
- [ ] Form feedback triggers
- [ ] Two-player cooperative platforming (Cargo Catch)

