use core::ports::input::TrackingError;
use core::pose::{Keypoint, Skeleton, KEYPOINT_COUNT};
use std::path::Path;
use std::sync::{Arc, OnceLock};

pub const MOVENET_INPUT_WIDTH: u32 = 192;
pub const MOVENET_INPUT_HEIGHT: u32 = 192;
pub const MOVENET_INPUT_LEN: usize =
    (MOVENET_INPUT_WIDTH as usize) * (MOVENET_INPUT_HEIGHT as usize) * 3;
pub const MOVENET_KEYPOINT_COUNT: usize = KEYPOINT_COUNT; // 17
pub const MOVENET_OUTPUT_LEN: usize = MOVENET_KEYPOINT_COUNT * 3; // 51 floats: [1, 1, 17, 3]

static ORT_ENV: OnceLock<Arc<ort::environment::Environment>> = OnceLock::new();

fn get_or_init_ort_env() -> Result<Arc<ort::environment::Environment>, TrackingError> {
    if let Some(env) = ORT_ENV.get() {
        return Ok(env.clone());
    }
    let env = ort::init()
        .with_name("MotionArcade-MoveNet")
        .build()
        .map_err(|e| {
            TrackingError::InferenceFailure(format!("Failed to build ORT environment: {}", e))
        })?;
    let arc = Arc::new(env);
    let _ = ORT_ENV.set(arc.clone());
    Ok(arc)
}

/// Parses a flat MoveNet [1, 1, 17, 3] output tensor into a domain `Skeleton`.
///
/// MoveNet landmark ordering matches COCO 17 keypoints:
/// [0: nose, 1: left_eye, 2: right_eye, 3: left_ear, 4: right_ear,
///  5: left_shoulder, 6: right_shoulder, 7: left_elbow, 8: right_elbow,
///  9: left_wrist, 10: right_wrist, 11: left_hip, 12: right_hip,
///  13: left_knee, 14: right_knee, 15: left_ankle, 16: right_ankle]
///
/// Output values per keypoint in MoveNet are [y, x, score].
/// Keypoints with score < `min_confidence` have their confidence clamped to 0.0.
pub fn parse_movenet_output(
    raw_output: &[f32],
    player_id: u8,
    min_confidence: f32,
) -> Result<Skeleton, TrackingError> {
    if raw_output.len() < MOVENET_OUTPUT_LEN {
        return Err(TrackingError::InferenceFailure(format!(
            "MoveNet output tensor too short: expected at least {} floats, got {}",
            MOVENET_OUTPUT_LEN,
            raw_output.len()
        )));
    }

    let mut keypoints = [Keypoint::ZERO; KEYPOINT_COUNT];
    for (i, kp) in keypoints.iter_mut().enumerate() {
        let base = i * 3;
        // In MoveNet output: index 0 = y, index 1 = x, index 2 = confidence score
        let y = raw_output.get(base).copied().unwrap_or(0.0).clamp(0.0, 1.0);
        let x = raw_output
            .get(base + 1)
            .copied()
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let score = raw_output
            .get(base + 2)
            .copied()
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);

        let confidence = if score >= min_confidence { score } else { 0.0 };
        *kp = Keypoint::new(x, y, confidence);
    }

    Ok(Skeleton::new(player_id, keypoints))
}

/// Parses a flat MoveNet [1, 1, 17, 3] output tensor without zeroing unconfident landmarks.
pub fn parse_movenet_output_raw(
    raw_output: &[f32],
    player_id: u8,
) -> Result<Skeleton, TrackingError> {
    parse_movenet_output(raw_output, player_id, 0.0)
}

/// ONNX Runtime MoveNet-Lightning neural inference engine.
pub struct MoveNetEngine {
    session: Option<ort::session::Session>,
    input_tensor_cache: Vec<f32>,
}

impl MoveNetEngine {
    /// Attempts to load an ONNX MoveNet model from the given path.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, TrackingError> {
        let path_ref = path.as_ref();
        if !path_ref.exists() {
            return Err(TrackingError::InferenceFailure(format!(
                "MoveNet ONNX model file does not exist: {:?}",
                path_ref
            )));
        }

        let env = get_or_init_ort_env()?;

        let session = ort::session::Session::builder(&env)
            .map_err(|e| {
                TrackingError::InferenceFailure(format!(
                    "Failed to create ORT session builder: {}",
                    e
                ))
            })?
            .commit_from_file(path_ref)
            .map_err(|e| {
                TrackingError::InferenceFailure(format!(
                    "Failed to commit ORT session from file: {}",
                    e
                ))
            })?;

        Ok(Self {
            session: Some(session),
            input_tensor_cache: vec![0.0; MOVENET_INPUT_LEN],
        })
    }

    /// Creates a headless / stub engine without an active ONNX session (for mock inference and parsing tests).
    pub fn new_mock() -> Self {
        Self {
            session: None,
            input_tensor_cache: vec![0.0; MOVENET_INPUT_LEN],
        }
    }

    /// Returns a mutable slice to the pre-allocated input tensor cache to prevent allocations in the 30 fps hot loop.
    pub fn input_buffer_mut(&mut self) -> &mut [f32] {
        &mut self.input_tensor_cache
    }

    /// Runs inference on the input tensor. If the engine is in mock mode (no session loaded),
    /// returns an error indicating real inference requires a committed model file.
    pub fn infer(
        &mut self,
        input_tensor: &[f32],
        player_id: u8,
        min_confidence: f32,
    ) -> Result<Skeleton, TrackingError> {
        let session = match self.session.as_mut() {
            Some(s) => s,
            None => {
                return Err(TrackingError::InferenceFailure(
                    "Cannot run inference: no ONNX model loaded".to_string(),
                ));
            }
        };

        if input_tensor.len() != MOVENET_INPUT_LEN {
            return Err(TrackingError::InferenceFailure(format!(
                "Input tensor size mismatch: expected {} floats, got {}",
                MOVENET_INPUT_LEN,
                input_tensor.len()
            )));
        }

        let shape = vec![
            1,
            MOVENET_INPUT_HEIGHT as i64,
            MOVENET_INPUT_WIDTH as i64,
            3,
        ];
        let tensor =
            ort::value::Tensor::from_array((shape, input_tensor.to_vec())).map_err(|e| {
                TrackingError::InferenceFailure(format!("Failed to create input tensor: {}", e))
            })?;

        let outputs = session.run(ort::inputs![tensor]).map_err(|e| {
            TrackingError::InferenceFailure(format!("Model execution failed: {}", e))
        })?;

        let first_output = outputs.values().next().ok_or_else(|| {
            TrackingError::InferenceFailure("No outputs returned from MoveNet model".to_string())
        })?;

        let (_, raw_data) = first_output.try_extract_tensor::<f32>().map_err(|e| {
            TrackingError::InferenceFailure(format!("Failed to extract output tensor: {}", e))
        })?;

        parse_movenet_output(raw_data, player_id, min_confidence)
    }
}
