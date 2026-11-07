# Contract 002: Camera Capture & ONNX Pose Inference Pipeline

**Contract ID:** `CONTRACT-INPUT-002`  
**Crate:** `crates/input`  
**Consumers:** `crates/app`  
**Guarantees:** Non-blocking async capture, worker inference thread, lock-free frame dropping on saturation.

---

## 1. Input Adapter Contract

```rust
use core::ports::input::{PlayerInput, PoseStreamPort, TrackingError};
use std::path::PathBuf;

pub struct CameraInferenceConfig {
    pub camera_index: u32,
    pub width: u32,
    pub height: u32,
    pub model_path: PathBuf,
    pub min_confidence: f32,
}

pub struct CameraPoseStreamAdapter {
    // Background worker handles nokhwa capture + ort MoveNet execution
    // Exposes lock-free receiver channel for latest PlayerInput
}

impl CameraPoseStreamAdapter {
    pub fn new(config: CameraInferenceConfig) -> Result<Self, TrackingError>;
}

impl PoseStreamPort for CameraPoseStreamAdapter {
    fn poll_input(&mut self) -> Result<Option<PlayerInput>, TrackingError> {
        // Non-blocking poll of latest skeleton
    }
}
```
