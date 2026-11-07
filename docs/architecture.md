# Architecture & Technical Pipeline

This document details the low-level processing pipeline, latency budgets, hardware setup options, and core data types for MotionArcade.

---

## 1. How It Works

```
 Camera ──► Pose model ──► Smoothing ──► Game loop ──► Renderer ──► Display
 (frames)   (keypoints)    (filtering)   (your game)   (graphics)   (TV/monitor/projector)
```

Each stage runs on its own thread and passes data through lock-free channels. A slow camera never blocks the renderer. A slow model never blocks the game loop.

### Pipeline Stages

| Stage | Job | Crate | Implementation Strategy |
|---|---|---|---|
| **Camera** | Low-latency 30 fps capture | `nokhwa` | Raw buffer access, zero-copy pointer to GPU texture |
| **Pose Model** | 17 keypoints per player | `ort` (ONNX Runtime) | MoveNet-Lightning / BlazePose on background thread |
| **Smoothing** | Remove jitter without lag | `core` | One Euro filter algorithm |
| **Game Loop** | Update state, collision math | Your game crate | Pure deterministic domain update |
| **Renderer** | **Native GPU Console Engine** | **`macroquad` + `miniquad` / `wgpu`** | **Direct VRAM texture streaming, custom GLSL/WGSL fragment shaders for neon player silhouettes, instanced particle FX** |
| **Audio** | Sound effects & spoken prompts | `rodio` | Multi-channel audio mixer |

---

## 2. Rendering Pipeline: Achieving "Nex Playground" Production Polish

To achieve consumer console fidelity (mirroring dedicated devices like Nex Playground), the rendering pipeline rejects sluggish webviews and IPC wrappers in favor of direct GPU rendering:

```
 Webcam Frame (nokhwa) ──► Zero-Copy DMA ──► GPU Texture (VRAM)
                                                  │
                                                  ▼
     Pose Keypoints (ort)  ──► Skeletal Bones ──► GLSL Fragment Shader (Neon Aura Mask)
                                                  │
                                                  ▼
     Game Objects (Orbs/Text) ──► Instanced Sprites + GPU Particles (Sparks/Punch)
                                                  │
                                                  ▼
                         Display Presentation (Locked 60 FPS / VSync)
```

### Why Native GPU Over Webviews:
1. **Zero IPC Latency:** Webviews add 16–30ms of serializing latency passing video buffers across JavaScript bridges. Direct GPU texture uploads take **< 1.5ms**.
2. **Custom Shaders:** GLSL/WGSL fragment shaders allow real-time glowing auras, comic hit-lines, bloom, and chromatic aberration directly over the child's silhouette.
3. **Hardware Acceleration Across Targets:** Identical native performance compiling for Linux, Windows, macOS, Android TV, and standard WebAssembly via WebGPU.

### Latency Budget & Frame Decoupling

Expect **100–150 ms** from physical movement to screen reaction:

| Segment | Time |
|---|---|
| Camera capture | ~33 ms (at 30 fps) |
| Pose inference | ~30–60 ms (MoveNet-Lightning on CPU) |
| One Euro smoothing | ~3 ms |
| Game + render | ~8 ms |
| Display lag | ~15–30 ms (varies by TV) |

#### Smart Rate Decoupling (For Chromebooks & Low-End CPUs)
On budget hardware (e.g. school Celeron processors), running ONNX pose inference at 30 fps can choke the CPU.
- **Inference Rate:** Decoupled to **15–20 fps** in a background worker thread.
- **Render & Game Rate:** Runs locked at **60 fps** using cubic/linear keypoint interpolation between inference ticks.
- **Result:** Reduces CPU compute load by ~40-50% while particle effects, animations, and camera motion remain buttery smooth.

---

## 2. Setting Up Your Play Space

### Display Options & Hardware Targets

| Setup | How | Latency | Effort | Best for |
|---|---|---|---|---|
| **Laptop + HDMI** | HDMI cable to TV, webcam on top | Lowest (~120 ms) | Low | POC, early adopters |
| **Browser (WebAssembly)** | Open a URL, use laptop/tablet camera | Medium (~160 ms) | **Zero** | Primary target: widest consumer & school reach |
| **Android TV box** | App installed on the box, USB webcam | Low (~130 ms) | Medium | Dedicated living-room product (Phase 2) |
| **Mini PC (Intel N100)** | Compact $100 box plugged into TV | Low (~110 ms) | Medium | Ideal turn-key appliance (faster & cheaper than Pi) |
| **Raspberry Pi 5 / AI Hat** | Linux native on Pi 5 (or Pi + Hailo NPU) | Medium (~140 ms) | High | DIY hobbyist & open hardware builds (Not for general parents) |
| **Phone → Chromecast** | Cast phone screen to TV | High (~250 ms+) | Low | Casual games only (latency penalty) |

### Spatial Calibration & "Living Room Floor Zone"

Real living rooms are chaotic: toddlers wander past, pets run by, and parents cross the frame.
- **Step-in Zone (Floor Plane Anchor):** During first-launch or round start, the game renders a glowing circular target on the floor (*"Step into the ring to activate!"*).
- **Spatial Bounding Cylinder:** Once a player steps in, the system establishes a 3D bounding cylinder around their torso scale.
- **Ghost & Phantom Filtering:** Any secondary skeleton appearing outside this boundary radius (or suddenly overlapping without entering from the edge) is automatically discarded.

---

## 3. Pose Data Reference

Games receive a `PlayerInput` every frame:

```rust
pub struct PlayerInput {
    pub players: Vec<Skeleton>,   // 0..=4 tracked people, smoothed
    pub timestamp: f64,           // seconds since session start
}

pub struct Skeleton {
    pub keypoints: [Keypoint; 17],
    pub id: u8,                   // stable ID across frames (player locking)
    pub body_scale: f32,          // estimated torso length in normalized coords
}

pub struct Keypoint {
    pub x: f32,            // 0.0 (left) to 1.0 (right), normalized to frame
    pub y: f32,            // 0.0 (top) to 1.0 (bottom)
    pub confidence: f32,   // 0.0 to 1.0
}
```

### Keypoint Helpers

Keypoint order follows the standard 17-point COCO layout:

| Helper | Returns |
|---|---|
| `nose()`, `left_wrist()`, `right_hip()` … | Individual keypoint |
| `torso_length()` | Shoulder-to-hip distance (for normalization) |
| `hand_direction(side)` | Angle of hand relative to chest center |
| `is_arms_up()` / `is_t_pose()` | Common pose checks |
| `hip_height_ratio()` | Hip height ÷ standing height (squat detection) |
| `is_confident(threshold)` | True if all major keypoints are above threshold |
| `zone(point, grid)` | Which zone (2×2, 3×3, etc.) a keypoint falls in |

---

## 4. Configuration & Calibration

Settings are persisted to `config.toml`:

```toml
[camera]
index = 0
width = 1280
height = 720

[tracking]
min_confidence = 0.3
smoothing = "one_euro"        # or "none" for raw keypoints
player_lock = true            # ignore new skeletons mid-game

[timing]
latency_offset_ms = 0         # set by calibration

[session]
break_after_minutes = 8       # suggest a break
max_session_minutes = 20      # hard stop (parent-configurable)

[display]
fullscreen = true
show_skeleton = false          # debug overlay
high_contrast = false

[network]
check_catalog_on_launch = true # false for fully offline mode
wifi_only_downloads = true
auto_update_packs = false      # parent must approve downloads
```

