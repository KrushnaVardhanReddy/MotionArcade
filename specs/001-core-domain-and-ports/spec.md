# Spec 001: Core Domain Traits & Hexagonal Ports

## Status
Approved / Ready for Implementation

---

## 1. Context & Motivation
MotionArcade must run across multiple targets: local native laptops, Android TV, browser WebAssembly (WASM), and automated headless CI. 
To guarantee this, the core domain must have **zero dependencies** on camera hardware (`nokhwa`), neural network engines (`ort`), or windowing frameworks (`macroquad`).

---

## 2. Inbound & Outbound Ports (Hexagonal Design)

```
                 [ Inbound Port: PoseStreamPort ]
                               │
               ┌───────────────┴───────────────┐
               ▼                               ▼
       CameraPoseStreamAdapter       MockFilePoseStreamAdapter
      (nokhwa + ort in `input`)     (Replays JSON in `core/tests`)
```

### Inbound Port: `PoseStreamPort`
```rust
pub trait PoseStreamPort: Send + 'static {
    fn poll_input(&mut self) -> Result<Option<PlayerInput>, TrackingError>;
}
```

### Outbound Port: `GameRendererPort` & `AudioPort`
```rust
pub trait GameCanvasPort {
    fn draw_circle(&mut self, x: f32, y: f32, radius: f32, color: [f32; 4]);
    fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32);
    fn draw_particles(&mut self, particles: &[Particle]);
}
```

---

## 3. Core Domain Models (`crates/core`)

1. **`Keypoint`**:
   - `x: f32` (0.0 to 1.0, normalized)
   - `y: f32` (0.0 to 1.0, normalized)
   - `confidence: f32` (0.0 to 1.0)
2. **`Skeleton`**:
   - `keypoints: [Keypoint; 17]` (standard COCO indices)
   - `id: u8` (stable tracking identity)
   - `body_scale: f32` (shoulder-to-hip torso distance)
3. **`PlayerInput`**:
   - `players: Vec<Skeleton>` (active players within the floor boundary)
   - `timestamp: f64` (monotonic seconds)

---

## 4. Domain Filters
- **One Euro Filter:** Dynamic low-pass jitter suppression that preserves high-speed responsiveness.
- **Floor Cylinder Boundary Filter:** Discards any skeleton whose root coordinates lie outside the calibrated player circle.

---

## 5. Acceptance Criteria
- [ ] `crates/core` compiles with zero dependencies on graphics or camera hardware.
- [ ] Unit tests verify One Euro smoothing reduces jitter by >60% on noisy mock sine-wave data.
- [ ] Floor cylinder filter successfully drops phantom skeletons outside the bounding radius.
- [ ] Zero unwrap guarantee: all error paths return `Result<T, CoreError>`.

