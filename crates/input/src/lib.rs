//! MotionArcade Camera & MoveNet Pose Inference Adapter
//!
//! Hexagonal inbound adapter connecting hardware camera capture (`nokhwa`),
//! bilinear frame preprocessing, and ONNX Runtime MoveNet model inference (`ort`),
//! streaming parsed 17-keypoint COCO skeletons through `core::ports::input::PoseStreamPort`.
//!
//! Guarantees:
//! - Zero `.unwrap()` or `.expect()` in non-test code.
//! - Non-blocking frame acquisition with lock-free saturation drop policy.
//! - Headless / CI resilience with seamless fallback to recorded mock streams or synthetic waves.

pub mod adapter;
pub mod config;
pub mod movenet;
pub mod preprocess;

pub use adapter::CameraPoseStreamAdapter;
pub use config::CameraInferenceConfig;
pub use movenet::{
    parse_movenet_output, parse_movenet_output_raw, MoveNetEngine, MOVENET_INPUT_HEIGHT,
    MOVENET_INPUT_LEN, MOVENET_INPUT_WIDTH, MOVENET_KEYPOINT_COUNT, MOVENET_OUTPUT_LEN,
};
pub use preprocess::Preprocessor;
