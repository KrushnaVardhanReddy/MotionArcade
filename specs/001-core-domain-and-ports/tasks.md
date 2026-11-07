# Tasks: Spec 001 — Core Domain Models, Hexagonal Ports & One Euro Filter

**Spec Reference:** [`specs/001-core-domain-and-ports/spec.md`](spec.md)  
**Quality Standards:** Zero `.unwrap()`, pure domain logic (no camera/graphics deps), deterministic unit tests.

---

## Task Breakdown

- [ ] **Task 1.1: Core Types & Keypoint Models**
  - Implement `Keypoint`, `Skeleton`, and `PlayerInput` in `crates/core/src/pose.rs`.
  - Add named joint helpers: `nose()`, `left_wrist()`, `right_wrist()`, `torso_length()`.
  - Add confidence validation helper `is_confident(threshold)`.

- [ ] **Task 1.2: One Euro Filter Smoothing**
  - Implement the One Euro filter algorithm in `crates/core/src/smoothing.rs`.
  - Provide configurable `min_cutoff`, `beta`, and `d_cutoff` parameters.
  - Implement dynamic jitter suppression benchmark test with synthetic sine wave data.

- [ ] **Task 1.3: Floor Cylinder Boundary Filter**
  - Implement spatial player anchoring in `crates/core/src/boundary.rs`.
  - Discard skeletons whose root position falls outside the calibrated radius.

- [ ] **Task 1.4: Hexagonal Ports Definition**
  - Define inbound port `PoseStreamPort` in `crates/core/src/ports/input.rs`.
  - Define outbound ports `GameCanvasPort` and `AudioPort` in `crates/core/src/ports/output.rs`.

- [ ] **Task 1.5: Mock Inbound Adapter**
  - Implement `MockFilePoseStreamAdapter` that reads recorded JSON pose sequences for headless CI.
  - Write test verifying mock streaming works without camera hardware.
