use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration options for physical camera capture and ONNX neural inference pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

