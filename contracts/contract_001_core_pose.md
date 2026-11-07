# Contract 001: Core Pose Domain & Hexagonal Input Stream

**Contract ID:** `CONTRACT-CORE-001`  
**Crate:** `crates/core`  
**Consumers:** `crates/input`, `crates/app`, `crates/games/*`  
**Guarantees:** Zero `.unwrap()`, zero I/O or graphics dependencies, thread-safe (`Send + Sync + 'static`).

---

## 1. Domain Types Contract

```rust
use serde::{Deserialize, Serialize};

/// Single 2D body landmark in normalized coordinate space [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Keypoint {
    /// Horizontal position: 0.0 (left edge of camera) to 1.0 (right edge).
    pub x: f32,
    /// Vertical position: 0.0 (top edge of camera) to 1.0 (bottom edge).
    pub y: f32,
    /// Confidence score from neural model: 0.0 (unseen) to 1.0 (certain).
    pub confidence: f32,
}

impl Keypoint {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, confidence: 0.0 };

    #[inline]
    pub fn is_confident(&self, threshold: f32) -> bool {
        self.confidence >= threshold
    }
}

/// Standard 17-point COCO pose skeleton.
/// Index Map:
/// 0: Nose, 1: Left Eye, 2: Right Eye, 3: Left Ear, 4: Right Ear,
/// 5: Left Shoulder, 6: Right Shoulder, 7: Left Elbow, 8: Right Elbow,
/// 9: Left Wrist, 10: Right Wrist, 11: Left Hip, 12: Right Hip,
/// 13: Left Knee, 14: Right Knee, 15: Left Ankle, 16: Right Ankle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Skeleton {
    /// Persistent player tracking identifier (1 to 4).
    pub id: u8,
    /// Normalized body scale: estimated distance between shoulder midpoint and hip midpoint.
    pub body_scale: f32,
    /// 17 COCO keypoints.
    pub keypoints: [Keypoint; 17],
}

impl Skeleton {
    pub fn nose(&self) -> Keypoint { self.keypoints[0] }
    pub fn left_shoulder(&self) -> Keypoint { self.keypoints[5] }
    pub fn right_shoulder(&self) -> Keypoint { self.keypoints[6] }
    pub fn left_wrist(&self) -> Keypoint { self.keypoints[9] }
    pub fn right_wrist(&self) -> Keypoint { self.keypoints[10] }
    pub fn left_hip(&self) -> Keypoint { self.keypoints[11] }
    pub fn right_hip(&self) -> Keypoint { self.keypoints[12] }

    /// Normalized shoulder-to-hip torso length for body-scale invariant hitboxes.
    pub fn torso_length(&self) -> f32 {
        let shoulder_y = (self.left_shoulder().y + self.right_shoulder().y) * 0.5;
        let hip_y = (self.left_hip().y + self.right_hip().y) * 0.5;
        (hip_y - shoulder_y).abs().max(0.1)
    }

    /// Check if hand is raised above nose level.
    pub fn is_hand_raised(&self, is_right: bool) -> bool {
        let hand = if is_right { self.right_wrist() } else { self.left_wrist() };
        hand.is_confident(0.3) && hand.y < self.nose().y
    }
}

/// Complete multi-player frame payload delivered to the game loop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerInput {
    /// Active filtered players inside the play boundary (0 to 4 players).
    pub players: Vec<Skeleton>,
    /// Monotonic timestamp in seconds since application start.
    pub timestamp: f64,
}
```

---

## 2. Inbound Port Contract (`PoseStreamPort`)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrackingError {
    #[error("Camera frame capture failed: {0}")]
    CaptureFailure(String),
    #[error("ONNX model inference failed: {0}")]
    InferenceFailure(String),
    #[error("Mock recording stream reached EOF")]
    StreamEnded,
    #[error("IO failure: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON deserialization failure: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Inbound hexagonal port consumed by game loop to pull real-time skeletons.
pub trait PoseStreamPort: Send + 'static {
    /// Poll the latest smoothed player input without blocking the render loop.
    fn poll_input(&mut self) -> Result<Option<PlayerInput>, TrackingError>;
}
```

---

## 3. One Euro Smoothing Filter Contract

```rust
/// Configurable adaptive low-pass filter for noisy landmark streams.
#[derive(Debug, Clone)]
pub struct OneEuroFilterConfig {
    /// Minimum cutoff frequency in Hz (higher = more jitter, less lag). Default: 1.0
    pub min_cutoff: f32,
    /// Speed coefficient parameter (higher = faster adaptation to high-speed movement). Default: 0.007
    pub beta: f32,
    /// Derivative cutoff frequency in Hz. Default: 1.0
    pub d_cutoff: f32,
}

pub struct OneEuroFilter {
    // Internal state: previous filtered position and derivative
}

impl OneEuroFilter {
    pub fn new(config: OneEuroFilterConfig) -> Self;
    pub fn filter(&mut self, value: f32, timestamp: f64) -> f32;
    pub fn reset(&mut self);
}
```

---

## 4. Verification Check
- `cargo test -p core` must pass with 0 warnings.
- No `unsafe`, no `.unwrap()`, no `.expect()` in non-test code.
