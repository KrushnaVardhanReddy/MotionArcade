# Spec 002: Real-Time Camera Capture & ONNX Pose Inference Adapter

## Status
Approved / Ready for Implementation

---

## 1. Context & Motivation
With `crates/core`, `crates/learning`, `crates/games/deflecto_bot`, and `crates/app` operating at 60 FPS on mock streams, Sprint 3 delivers the live hardware bridge:
Capturing physical camera frames using `nokhwa`, resizing to a 192x192 RGB tensor, running MoveNet-Lightning via ONNX Runtime (`ort`), and streaming parsed 17-keypoint skeletons through `PoseStreamPort`.

---

## 2. Hexagonal Architecture & Threading

```
 ┌──────────────────────────────────────────────────────────────┐
 │                crates/input (Inbound Adapter)                │
 │                                                              │
 │   Camera Capture Thread           Inference Worker Thread    │
 │   ─────────────────────           ───────────────────────    │
 │   nokhwa (30 FPS)                 ort (MoveNet-Lightning)    │
 │          │                                   │               │
 │          ▼                                   ▼               │
 │   [Raw RGB Frame] ──► Bounded Ring ──► [192x192 Tensor]      │
 │                       (Drop Policy)          │               │
 │                                              ▼               │
 │                                      [Parsed Skeleton]       │
 │                                              │               │
 └──────────────────────────────────────────────┼───────────────┘
                                                ▼
                                    crates/core::PoseStreamPort
                                                ▼
                                           crates/app
```

### Frame Drop Policy (Zero-Lag Guarantee)
- If the inference thread is processing a frame, newer camera frames replace the pending frame in the single-slot ring buffer.
- Under heavy load, frame rate drops slightly, but **latency never accumulates**. Skeletons always reflect the absolute freshest frame.

---

## 3. MoveNet-Lightning 192x192 Output Specification
- **Input:** Single image tensor `[1, 3, 192, 192]` float32 normalized to `[0.0, 1.0]`.
- **Output:** Tensor shape `[1, 1, 17, 3]`.
  - For each of the 17 keypoints:
    - Index 0: `y` coordinate [0.0, 1.0]
    - Index 1: `x` coordinate [0.0, 1.0]
    - Index 2: `confidence` [0.0, 1.0]

---

## 4. Acceptance Criteria
- [ ] `crates/input` implements `core::ports::input::PoseStreamPort`.
- [ ] Camera capture gracefully falls back to mock/synthetic stream if no physical camera or display is present.
- [ ] ONNX model tensor memory is pre-allocated; zero dynamic buffer allocations in the 30 fps hot loop.
- [ ] Zero `.unwrap()` in production code.
