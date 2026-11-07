# MotionArcade Gemini Pair & Rust Engineering Standards

All agents and contributors implementing Rust code across MotionArcade (`core`, `input`, `learning`, `app`, `games/*`) MUST strictly adhere to the following principles.

---

### 1. Zero Panic Guarantee (Error Handling)
- **NEVER** use `.unwrap()` or `.expect()` in non-test production code. A dropped camera frame, noisy pose coordinate, or missing audio file must **never** crash a classroom lesson or living-room play session.
- Always propagate errors via `Result<T, E>` using `?` or handle them gracefully with `match` / `if let`.
- Use `thiserror` for domain and pipeline error types (`PipelineError`, `TrackingError`, `ContentPackError`).
- Use `tracing` for structured logs (`tracing::warn!`, `tracing::error!`).

---

### 2. Hexagonal Architecture (Ports & Adapters)
- **Pure Core Engine (`crates/core`)**:
  - Contains strictly domain logic: 17-keypoint models, One Euro smoothing filter, floor plane bounding cylinder filters, the Leitner spaced repetition engine, and the `Game` trait.
  - Zero camera dependencies, zero OS windowing dependencies, zero async runtime.
  - Fully testable in headless CI with zero real hardware.
- **Inbound Adapters (`crates/input`)**:
  - Real camera capture (`nokhwa`) and ONNX inference (`ort`) are isolated in `input`.
  - Replaced cleanly during tests via recorded mock files (`--mock-input`).
- **Outbound Adapters (`crates/app`, `crates/learning`)**:
  - `macroquad` (rendering) and `rodio` (audio) interact with `core` through well-defined contexts (`GameContext`).

---

### 3. Rate Decoupling & Threading Performance
- **Inference Thread:** Camera capture and ONNX neural network inference run on their own thread at **15–30 fps**.
- **Game / Render Thread:** The game update loop and rendering run locked at **60 fps**, interpolating keypoints between inference ticks.
- **Lock-Free Communication:** Crossbeam or standard bounded mpsc channels pass keypoint snapshots between threads; a slow pose inference tick must never cause UI stutter or audio pop.

---

### 4. Zero Cloud Policy & 100% Offline-First
- Never introduce outbound network connections or remote telemetry in the core app.
- Content packs, pose models, audio, and configurations must be 100% resolvable from local filesystem paths.
- All student progress and profiles remain strictly local on-device.

---

### 5. Automated Pre-Submission Verification
Before committing or submitting changes:
1. `cargo check --workspace` — verify all workspace crates compile cleanly.
2. `cargo fmt --check` — verify Rust code formatting.
3. `cargo clippy --workspace --all-targets -- -D warnings` — zero clippy warnings.
4. `cargo test --workspace` — all unit and integration tests must pass.
5. `cargo run --release -- --mock-input` — verify simulated game loop runs with 0 panics.

---

### 6. Inspectable Evidence & Reports
- Performance benchmarks and pipeline assertions must record machine-readable receipts into `reports/<task-name>/`:
  - `verification.json`: FPS, latency, and keypoint confidence metrics.
  - Summary report in `reports/<TASK_NAME>_REPORT.md`.
