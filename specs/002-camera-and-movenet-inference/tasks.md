# Tasks: Spec 002 — Camera Capture & ONNX Pose Inference Adapter

**Spec Reference:** [`specs/002-camera-and-movenet-inference/spec.md`](spec.md)  
**Contract Reference:** [`contracts/contract_002_camera_inference.md`](../../contracts/contract_002_camera_inference.md)  
**Quality Standards:** Zero `.unwrap()`, thread-safe, rate-decoupled, zero latency accumulation.

---

## Task Breakdown

- [ ] **Task 2.1: Image Preprocessing & Tensor Conversion**
  - Implement fast nearest/bilinear RGB frame downscaling to 192x192 in `crates/input/src/preprocess.rs`.
  - Normalize uint8 pixel values `[0, 255]` to float32 `[0.0, 1.0]`.

- [ ] **Task 2.2: MoveNet ONNX Inference Engine (`ort`)**
  - Implement `MoveNetEngine` in `crates/input/src/movenet.rs`.
  - Parse the `[1, 1, 17, 3]` output tensor into `core::pose::Skeleton`.
  - Apply confidence thresholding and assign stable tracking IDs.

- [ ] **Task 2.3: Physical Camera Capture Adapter (`nokhwa`)**
  - Implement `CameraSource` in `crates/input/src/camera.rs`.
  - Manage camera device handle and non-blocking frame polling.
  - Gracefully fallback to mock streaming if camera device open fails.

- [ ] **Task 2.4: PoseStreamPort Implementation & Threading Bridge**
  - Implement `CameraPoseStreamAdapter` in `crates/input/src/adapter.rs`.
  - Bridge camera frames and inference worker via crossbeam/bounded channels.
  - Implement `PoseStreamPort` for consumption by `crates/app`.

- [ ] **Task 2.5: Integration & Verification Tests**
  - Add mock tensor inference unit tests with known synthetic keypoint coordinates.
  - Verify zero clippy warnings and 100% test pass rate.
