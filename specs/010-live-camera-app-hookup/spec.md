# Spec 010: Live Hardware Camera Integration in App Runner

## Status
Approved / Ready for Parallel Implementation

---

## 1. Context & Motivation
Currently `crates/app/src/main.rs` reads from synthetic waving or a mock file.
Spec 010 wires `crates/input::CameraPoseStreamAdapter` directly into `crates/app`:
- Auto-detects if a physical camera is connected and if `assets/models/movenet_lightning.onnx` exists.
- If present, initializes real-time 30 FPS webcam capture + MoveNet inference, streaming live 17-keypoint skeletons to the game loop.
- If absent, logs an informative notice and falls back to mock/synthetic input so headless tests and machines without cameras continue running smoothly.
- Exposes CLI flags: `--camera <index>`, `--no-camera`, `--model <path>`, `--mirror`.

---

## 2. Zero-Conflict File Scope
- Operates strictly in:
  - `crates/app/Cargo.toml` (add `input = { path = "../input" }`)
  - `crates/app/src/lib.rs` (export camera pose source helper)
  - `crates/app/src/main.rs` (initialize `CameraPoseStreamAdapter`)
  - `crates/app/tests/game_loop_test.rs`
- Zero edits to `content/` or `crates/learning/` (reserved for Agent 2).

---

## 3. Acceptance Criteria
- [ ] Compiles and passes `cargo test -p app`.
- [ ] If no camera is present, runs without panic via headless fallback.
- [ ] Exactly zero `.unwrap()` in production code.
