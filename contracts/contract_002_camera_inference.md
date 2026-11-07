# Contract 002: Camera Capture & ONNX Pose Inference Pipeline

**Contract ID:** `CONTRACT-INPUT-002`  
**Crate:** `crates/input`  
**Consumers:** `crates/app`  
**Guarantees:** Non-blocking async capture, worker inference thread, lock-free frame dropping on saturation, zero `.unwrap()`.

---

## 1. Public Configuration Contract

```rust
use std::path::PathBuf;

/// Configuration options for the physical camera capture and ONNX neural inference pipeline.
#[derive(Debug, Clone)]
pub struct CameraInferenceConfig {
    /// Device index (default: 0 for primary webcam).
    pub camera_index: u32,
    /// Native capture width in pixels (e.g., 640 or 1280).
    pub capture_width: u32,
    /// Native capture height in pixels (e.g., 480 or 720).
    pub capture_height: u32,
    /// Target frame rate in FPS (default: 30).
    pub fps: u32,
    /// Path to the MoveNet ONNX model file on disk.
    pub model_path: PathBuf,
    /// Minimum keypoint confidence threshold (e.g., 0.25).
    pub min_confidence: f32,
    /// Whether to mirror the camera horizontally (default: true for natural mirror game feel).
    pub mirror_horizontal: bool,
}

impl Default for CameraInferenceConfig {
    fn default() -> Self {
        Self {
            camera_index: 0,
            capture_width: 640,
            capture_height: 480,
            fps: 30,
            model_path: PathBuf::from("assets/models/movenet_lightning.onnx"),
            min_confidence: 0.25,
            mirror_horizontal: true,
        }
    }
}
```

---

## 2. Public Adapter Contract (`PoseStreamPort`)

```rust
use core::ports::input::{PlayerInput, PoseStreamPort, TrackingError};
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;

/// Live camera pose stream adapter implementing the hexagonal inbound port `PoseStreamPort`.
pub struct CameraPoseStreamAdapter {
    rx: Receiver<PlayerInput>,
    _worker: JoinHandle<()>,
}

impl CameraPoseStreamAdapter {
    /// Spawns the camera capture and ONNX inference worker threads.
    /// If camera access fails, returns `TrackingError::CaptureFailure`.
    pub fn new(config: CameraInferenceConfig) -> Result<Self, TrackingError>;
    
    /// Spawns with fallback: if camera open fails, falls back to synthetic waving or mock file.
    pub fn with_mock_fallback(config: CameraInferenceConfig, fallback_mock_path: Option<PathBuf>) -> Self;
}

impl PoseStreamPort for CameraPoseStreamAdapter {
    /// Non-blocking poll for the latest player input. Returns `Ok(None)` if no new frame has arrived.
    fn poll_input(&mut self) -> Result<Option<PlayerInput>, TrackingError>;
}
```

---

## 3. Preprocessing Contract

```rust
pub struct Preprocessor {
    target_width: u32,  // 192 for MoveNet Lightning
    target_height: u32, // 192
}

impl Preprocessor {
    pub fn new(target_width: u32, target_height: u32) -> Self;

    /// Downscales raw RGB bytes into a flat float32 array normalized to [0.0, 1.0].
    /// Output buffer length: target_width * target_height * 3
    pub fn process_rgb8(
        &self,
        src_bytes: &[u8],
        src_width: u32,
        src_height: u32,
        mirror_horizontal: bool,
        out_tensor: &mut [f32],
    ) -> Result<(), TrackingError>;
}
```

---

## 4. Verification Check
- `cargo test -p input` passes with 0 warnings.
- `cargo clippy -p input --all-targets -- -D warnings` clean.
- Unit tests run in headless environments with zero physical webcam or GPU requirements.
