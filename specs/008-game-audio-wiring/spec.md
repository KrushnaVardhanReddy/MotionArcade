# Spec 008: Game Audio Wiring & Synthetic Haptics Integration

## Status
Approved / Ready for Parallel Implementation

---

## 1. Context & Motivation
Currently `crates/audio` has tested sound effects and procedural synthesis (`DeflectChime { pan }`, `BananaChomp`, `CorrectFanfare`, `GentleWobble`), but the game crates (`deflecto_bot` and `banana_grab`) don't yet call it during collisions.
Spec 008 connects the non-blocking audio mixer into both games:
- Deflecto-Bot hits trigger `DeflectChime` with stereo panning based on the struck quadrant (-0.75 for left quadrants, +0.75 for right).
- Incorrect deflections trigger `GentleWobble`.
- Completing a question wave triggers `CorrectFanfare`.
- Banana Grab fruit catches trigger `BananaChomp`.

---

## 2. Zero-Conflict Crate Isolation
- Operates strictly in:
  - `crates/games/deflecto_bot/Cargo.toml` & `crates/games/deflecto_bot/src/lib.rs`
  - `crates/games/banana_grab/Cargo.toml` & `crates/games/banana_grab/src/lib.rs`
- Does not modify `crates/app/src/main.rs` (which is reserved for Agent 2).

---

## 3. Acceptance Criteria
- [ ] `deflecto_bot` and `banana_grab` take an optional `Arc<AudioMixer>`.
- [ ] Deflecting orbs / catching bananas triggers the non-blocking audio channel.
- [ ] Headless unit tests pass with audio wired.
- [ ] Zero `.unwrap()` in production code.
