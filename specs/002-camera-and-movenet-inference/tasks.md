# Tasks: Spec 002 — Camera Capture & ONNX Pose Inference

**Spec Reference:** [`specs/002-camera-and-movenet-inference/spec.md`](spec.md)  
**Quality Standards:** Background worker thread, rate decoupling, zero latency buildup.

---

## Task Breakdown

- [ ] **Task 2.1: Camera Inbound Adapter (`nokhwa`)**
  - Implement `CameraAdapter` in `crates/input/src/camera.rs`.
  - Capture RGB frames at 30 fps using non-blocking capture polling.

- [ ] **Task 2.2: ONNX Runtime MoveNet Inference Engine (`ort`)**
  - Implement `MoveNetEngine` in `crates/input/src/movenet.rs`.
  - Pre-allocate 192x192 RGB input tensor to avoid allocations in hot loop.
  - Parse 17 keypoints + confidence scores into domain `Skeleton`.

- [ ] **Task 2.3: Frame Decoupling & Drop Policy**
  - Pass frames across lock-free bounded channel.
  - Implement drop policy if inference worker is saturated, ensuring zero latency lag.

- [ ] **Task 2.4: Integrate with `PoseStreamPort`**
  - Implement `PoseStreamPort` for `crates/input`.
  - Provide fallback to mock adapter if no webcam is detected.
