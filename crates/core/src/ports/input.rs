use crate::pose::PlayerInput;
use thiserror::Error;

/// Error conditions that can occur during pose stream ingestion and inference.
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

/// Inbound hexagonal port consumed by the game loop to poll real-time skeletons.
pub trait PoseStreamPort: Send + 'static {
    /// Poll the latest smoothed player input without blocking the render loop.
    ///
    /// Returns:
    /// - `Ok(Some(input))` when a new frame is ready
    /// - `Ok(None)` if no new frame is currently available (non-blocking)
    /// - `Err(TrackingError::StreamEnded)` when a playback stream finishes
    /// - `Err(...)` if a capture or inference failure occurred
    fn poll_input(&mut self) -> Result<Option<PlayerInput>, TrackingError>;
}
