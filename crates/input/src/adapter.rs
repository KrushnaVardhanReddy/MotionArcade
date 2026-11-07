use crate::config::CameraInferenceConfig;
use crate::movenet::{MoveNetEngine, MOVENET_INPUT_HEIGHT, MOVENET_INPUT_LEN, MOVENET_INPUT_WIDTH};
use crate::preprocess::Preprocessor;
use core::ports::input::{PlayerInput, PoseStreamPort, TrackingError};
use core::pose::{
    Keypoint, Skeleton, COCO_LEFT_ANKLE, COCO_LEFT_EAR, COCO_LEFT_ELBOW, COCO_LEFT_EYE,
    COCO_LEFT_HIP, COCO_LEFT_KNEE, COCO_LEFT_SHOULDER, COCO_LEFT_WRIST, COCO_NOSE,
    COCO_RIGHT_ANKLE, COCO_RIGHT_EAR, COCO_RIGHT_ELBOW, COCO_RIGHT_EYE, COCO_RIGHT_KNEE,
    COCO_RIGHT_SHOULDER, COCO_RIGHT_WRIST, KEYPOINT_COUNT,
};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Live camera pose stream adapter implementing hexagonal inbound port `PoseStreamPort`.
pub struct CameraPoseStreamAdapter {
    rx: Receiver<PlayerInput>,
    _worker: JoinHandle<()>,
}

impl CameraPoseStreamAdapter {
    /// Spawns the camera capture and ONNX inference worker threads.
    /// If camera access or ONNX model initialization fails, returns `TrackingError`.
    pub fn new(config: CameraInferenceConfig) -> Result<Self, TrackingError> {
        let (init_tx, init_rx) = channel::<Result<(), TrackingError>>();
        let (data_tx, data_rx) = sync_channel::<PlayerInput>(2);

        let worker_config = config.clone();
        let worker = std::thread::Builder::new()
            .name("movenet-capture-worker".to_string())
            .spawn(move || {
                let camera_index = CameraIndex::Index(worker_config.camera_index);
                let requested_format = RequestedFormat::new::<RgbFormat>(
                    RequestedFormatType::Closest(CameraFormat::new_from(
                        worker_config.capture_width,
                        worker_config.capture_height,
                        FrameFormat::RAWRGB,
                        worker_config.fps,
                    )),
                );

                let mut camera = match Camera::new(camera_index, requested_format) {
                    Ok(cam) => cam,
                    Err(e) => {
                        let _ = init_tx.send(Err(TrackingError::CaptureFailure(format!(
                            "Failed to open camera device: {}",
                            e
                        ))));
                        return;
                    }
                };

                if let Err(e) = camera.open_stream() {
                    let _ = init_tx.send(Err(TrackingError::CaptureFailure(format!(
                        "Failed to open camera stream: {}",
                        e
                    ))));
                    return;
                }

                let mut engine = match MoveNetEngine::from_file(&worker_config.model_path) {
                    Ok(eng) => eng,
                    Err(e) => {
                        let _ = init_tx.send(Err(e));
                        return;
                    }
                };

                // Signal initialization success
                let _ = init_tx.send(Ok(()));

                run_live_capture_loop(camera, &mut engine, worker_config, data_tx);
            })
            .map_err(|e| {
                TrackingError::CaptureFailure(format!("Failed to spawn worker thread: {}", e))
            })?;

        match init_rx.recv() {
            Ok(Ok(())) => Ok(Self {
                rx: data_rx,
                _worker: worker,
            }),
            Ok(Err(err)) => Err(err),
            Err(_) => Err(TrackingError::CaptureFailure(
                "Worker thread terminated during initialization".to_string(),
            )),
        }
    }

    /// Spawns with fallback: if camera open or model load fails, falls back to pre-recorded mock file or synthetic wave.
    pub fn with_mock_fallback(
        config: CameraInferenceConfig,
        fallback_mock_path: Option<PathBuf>,
    ) -> Self {
        match Self::new(config.clone()) {
            Ok(adapter) => adapter,
            Err(err) => {
                tracing::warn!(
                    "Camera initialization unavailable ({}). Engaging mock/synthetic fallback stream.",
                    err
                );
                spawn_fallback_stream(config, fallback_mock_path)
            }
        }
    }
}

impl PoseStreamPort for CameraPoseStreamAdapter {
    /// Non-blocking poll for the latest player input. Returns `Ok(None)` if no new frame has arrived.
    fn poll_input(&mut self) -> Result<Option<PlayerInput>, TrackingError> {
        let mut latest = None;
        loop {
            match self.rx.try_recv() {
                Ok(frame) => latest = Some(frame),
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    if latest.is_some() {
                        break;
                    }
                    return Err(TrackingError::StreamEnded);
                }
            }
        }
        Ok(latest)
    }
}

/// Hot loop running real physical camera capture and ONNX inference at target FPS.
fn run_live_capture_loop(
    mut camera: Camera,
    engine: &mut MoveNetEngine,
    config: CameraInferenceConfig,
    tx: SyncSender<PlayerInput>,
) {
    let preprocessor = Preprocessor::new(MOVENET_INPUT_WIDTH, MOVENET_INPUT_HEIGHT);
    let mut normalized_tensor = vec![0.0f32; MOVENET_INPUT_LEN];
    let start_instant = Instant::now();
    let tick_interval = Duration::from_millis((1000 / config.fps.max(1)) as u64);

    while camera.is_stream_open() {
        let frame_start = Instant::now();

        let frame_buffer = match camera.frame() {
            Ok(buf) => buf,
            Err(e) => {
                tracing::warn!("Camera frame capture dropped: {}", e);
                std::thread::sleep(Duration::from_millis(5));
                continue;
            }
        };

        let rgb_image = match frame_buffer.decode_image::<RgbFormat>() {
            Ok(img) => img,
            Err(e) => {
                tracing::warn!("Failed to decode RGB frame: {}", e);
                continue;
            }
        };

        if let Err(e) = preprocessor.process_rgb8(
            &rgb_image,
            rgb_image.width(),
            rgb_image.height(),
            config.mirror_horizontal,
            &mut normalized_tensor,
        ) {
            tracing::warn!("Frame preprocessing failed: {}", e);
            continue;
        }

        let skeleton = match engine.infer(&normalized_tensor, 1, config.min_confidence) {
            Ok(skel) => skel,
            Err(e) => {
                tracing::warn!("MoveNet inference step failed: {}", e);
                continue;
            }
        };

        let timestamp = start_instant.elapsed().as_secs_f64();
        let payload = PlayerInput::new(vec![skeleton], timestamp);

        // Lock-free non-blocking send: if buffer is full, drop older frame to avoid latency lag
        let _ = tx.try_send(payload);

        let elapsed = frame_start.elapsed();
        if elapsed < tick_interval {
            std::thread::sleep(tick_interval - elapsed);
        }
    }
}

/// Fallback stream provider spawning either pre-recorded mock stream or procedural synthetic wave.
fn spawn_fallback_stream(
    config: CameraInferenceConfig,
    mock_path: Option<PathBuf>,
) -> CameraPoseStreamAdapter {
    let (tx, rx) = sync_channel::<PlayerInput>(4);

    let worker = std::thread::Builder::new()
        .name("input-fallback-worker".to_string())
        .spawn(move || {
            let tick_interval = Duration::from_millis((1000 / config.fps.max(1)) as u64);

            if let Some(path) = mock_path {
                if path.exists() {
                    tracing::info!("Starting mock recording replay from {:?}", path);
                    run_mock_file_loop(&path, tick_interval, &tx);
                    return;
                } else {
                    tracing::warn!(
                        "Mock file {:?} not found, falling back to synthetic wave.",
                        path
                    );
                }
            }

            tracing::info!("Starting real-time procedural synthetic wave generator.");
            run_synthetic_wave_loop(tick_interval, &tx);
        })
        .unwrap_or_else(|_| std::thread::spawn(|| {}));

    CameraPoseStreamAdapter {
        rx,
        _worker: worker,
    }
}

/// Reads pre-recorded JSON lines file in an infinite loop, streaming PlayerInput at fixed intervals.
fn run_mock_file_loop(path: &Path, tick_interval: Duration, tx: &SyncSender<PlayerInput>) {
    loop {
        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) => {
                tracing::error!("Could not open mock file {:?}: {}", path, e);
                break;
            }
        };

        let reader = std::io::BufReader::new(file);
        for line_result in reader.lines() {
            let start = Instant::now();
            let line = match line_result {
                Ok(l) => l,
                Err(_) => break,
            };

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Ok(input) = serde_json::from_str::<PlayerInput>(trimmed) {
                if tx.send(input).is_err() {
                    return; // Receiver disconnected, exit thread
                }
            }

            let elapsed = start.elapsed();
            if elapsed < tick_interval {
                std::thread::sleep(tick_interval - elapsed);
            }
        }
    }
}

/// Procedural synthetic waving player generator delivering frames at target interval.
fn run_synthetic_wave_loop(tick_interval: Duration, tx: &SyncSender<PlayerInput>) {
    let start_instant = Instant::now();
    loop {
        let frame_start = Instant::now();
        let elapsed_secs = start_instant.elapsed().as_secs_f64();
        let input = generate_synthetic_wave(elapsed_secs);

        if tx.send(input).is_err() {
            return; // Receiver disconnected
        }

        let elapsed = frame_start.elapsed();
        if elapsed < tick_interval {
            std::thread::sleep(tick_interval - elapsed);
        }
    }
}

/// Generates a procedural 17-keypoint COCO player skeleton waving hands for test & simulation.
pub fn generate_synthetic_wave(timestamp: f64) -> PlayerInput {
    let t = timestamp as f32;
    let mut keypoints = [Keypoint::ZERO; KEYPOINT_COUNT];

    // Head
    let nose_x = 0.50;
    let nose_y = 0.22;
    keypoints[COCO_NOSE] = Keypoint::new(nose_x, nose_y, 0.98);
    keypoints[COCO_LEFT_EYE] = Keypoint::new(nose_x - 0.025, nose_y - 0.02, 0.95);
    keypoints[COCO_RIGHT_EYE] = Keypoint::new(nose_x + 0.025, nose_y - 0.02, 0.95);
    keypoints[COCO_LEFT_EAR] = Keypoint::new(nose_x - 0.055, nose_y, 0.92);
    keypoints[COCO_RIGHT_EAR] = Keypoint::new(nose_x + 0.055, nose_y, 0.92);

    // Shoulders
    let ls_x = 0.42;
    let ls_y = 0.33;
    let rs_x = 0.58;
    let rs_y = 0.33;
    keypoints[COCO_LEFT_SHOULDER] = Keypoint::new(ls_x, ls_y, 0.96);
    keypoints[COCO_RIGHT_SHOULDER] = Keypoint::new(rs_x, rs_y, 0.96);

    // Left arm: dynamic wave
    let lw_x = 0.26 + (t * 2.2).sin() * 0.15;
    let lw_y = 0.30 - (t * 1.8).cos() * 0.18;
    let le_x = (ls_x + lw_x) * 0.5 - 0.03;
    let le_y = (ls_y + lw_y) * 0.5;
    keypoints[COCO_LEFT_ELBOW] = Keypoint::new(le_x, le_y, 0.94);
    keypoints[COCO_LEFT_WRIST] = Keypoint::new(lw_x, lw_y, 0.96);

    // Right arm: dynamic wave
    let rw_x = 0.74 - (t * 2.4).cos() * 0.15;
    let rw_y = 0.30 - (t * 2.0).sin() * 0.18;
    let re_x = (rs_x + rw_x) * 0.5 + 0.03;
    let re_y = (rs_y + rw_y) * 0.5;
    keypoints[COCO_RIGHT_ELBOW] = Keypoint::new(re_x, re_y, 0.94);
    keypoints[COCO_RIGHT_WRIST] = Keypoint::new(rw_x, rw_y, 0.96);

    // Hips
    let lh_x = 0.46;
    let lh_y = 0.58;
    let rh_x = 0.54;
    let rh_y = 0.58;
    keypoints[COCO_LEFT_HIP] = Keypoint::new(lh_x, lh_y, 0.95);
    keypoints[core::pose::COCO_RIGHT_HIP] = Keypoint::new(rh_x, rh_y, 0.95);

    // Knees
    let lk_x = 0.45;
    let lk_y = 0.76;
    let rk_x = 0.55;
    let rk_y = 0.76;
    keypoints[COCO_LEFT_KNEE] = Keypoint::new(lk_x, lk_y, 0.92);
    keypoints[COCO_RIGHT_KNEE] = Keypoint::new(rk_x, rk_y, 0.92);

    // Ankles
    let la_x = 0.44;
    let la_y = 0.92;
    let ra_x = 0.56;
    let ra_y = 0.92;
    keypoints[COCO_LEFT_ANKLE] = Keypoint::new(la_x, la_y, 0.90);
    keypoints[COCO_RIGHT_ANKLE] = Keypoint::new(ra_x, ra_y, 0.90);

    let skel = Skeleton::new(1, keypoints);
    PlayerInput::new(vec![skel], timestamp)
}
