# Spec 002: Camera Capture & ONNX Pose Inference

## Status
Draft / In Design

---

## 1. Goal
Provide an asynchronous inbound adapter implementing `PoseStreamPort` using `nokhwa` for camera frame capture and `ort` (ONNX Runtime) for running MoveNet-Lightning (192x192).

---

## 2. Threading & Channel Decoupling
- Camera capture thread operates at device rate (~30 fps).
- Worker thread executes ONNX inference (`ort`) using a pre-allocated input tensor.
- Outputs `Skeleton` with 17 normalized keypoints into a crossbeam ring channel.
- If inference takes >50ms, older unconsumed frames are dropped to prevent latency buildup.

---

## 3. Acceptance Criteria
- [ ] Loads `movenet_lightning.onnx` from `assets/models/`.
- [ ] Feeds 30 fps RGB frames into inference loop without memory leaks or allocations in the hot loop.
- [ ] Implements `PoseStreamPort` cleanly.

