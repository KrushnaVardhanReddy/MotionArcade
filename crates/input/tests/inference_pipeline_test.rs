use core::ports::input::PoseStreamPort;
use core::pose::{COCO_LEFT_ANKLE, COCO_LEFT_WRIST, COCO_NOSE, COCO_RIGHT_WRIST};
use input::adapter::CameraPoseStreamAdapter;
use input::config::CameraInferenceConfig;
use input::movenet::{
    parse_movenet_output, parse_movenet_output_raw, MOVENET_INPUT_HEIGHT, MOVENET_INPUT_LEN,
    MOVENET_INPUT_WIDTH, MOVENET_OUTPUT_LEN,
};
use input::preprocess::Preprocessor;
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn test_camera_inference_config_defaults() {
    let config = CameraInferenceConfig::default();
    assert_eq!(config.camera_index, 0);
    assert_eq!(config.capture_width, 640);
    assert_eq!(config.capture_height, 480);
    assert_eq!(config.fps, 30);
    assert_eq!(
        config.model_path,
        PathBuf::from("assets/models/movenet_lightning.onnx")
    );
    assert!((config.min_confidence - 0.25).abs() < 1e-5);
    assert!(config.mirror_horizontal);
}

#[test]
fn test_preprocessor_bilinear_resizing_and_mirroring() {
    let preprocessor = Preprocessor::new(2, 2);
    assert_eq!(preprocessor.target_width(), 2);
    assert_eq!(preprocessor.target_height(), 2);
    assert_eq!(preprocessor.target_len(), 12);

    // 4x4 image where left two columns are pure Red (255, 0, 0) and right two columns are pure Blue (0, 0, 255)
    let src_w = 4u32;
    let src_h = 4u32;
    let mut src_bytes = vec![0u8; (src_w * src_h * 3) as usize];
    for y in 0..src_h {
        for x in 0..src_w {
            let idx = ((y * src_w + x) * 3) as usize;
            if x < 2 {
                src_bytes[idx] = 255; // Red
                src_bytes[idx + 1] = 0;
                src_bytes[idx + 2] = 0;
            } else {
                src_bytes[idx] = 0;
                src_bytes[idx + 1] = 0;
                src_bytes[idx + 2] = 255; // Blue
            }
        }
    }

    // Process without horizontal mirroring: left should stay Red, right should stay Blue
    let mut out_normal = vec![0.0f32; 12];
    let res = preprocessor.process_rgb8(&src_bytes, src_w, src_h, false, &mut out_normal);
    assert!(res.is_ok());

    // Top-left pixel (x=0, y=0) should be Red
    assert!(out_normal[0] > 0.8, "Normal top-left should be Red R");
    assert!(
        out_normal[2] < 0.2,
        "Normal top-left should have low Blue B"
    );

    // Top-right pixel (x=1, y=0) should be Blue
    assert!(
        out_normal[3] < 0.2,
        "Normal top-right should have low Red R"
    );
    assert!(out_normal[5] > 0.8, "Normal top-right should be Blue B");

    // Process WITH horizontal mirroring: left and right columns must be inverted
    let mut out_mirrored = vec![0.0f32; 12];
    let res = preprocessor.process_rgb8(&src_bytes, src_w, src_h, true, &mut out_mirrored);
    assert!(res.is_ok());

    // Top-left pixel of mirrored should now be Blue
    assert!(
        out_mirrored[0] < 0.2,
        "Mirrored top-left should have low Red"
    );
    assert!(out_mirrored[2] > 0.8, "Mirrored top-left should be Blue");

    // Top-right pixel of mirrored should now be Red
    assert!(out_mirrored[3] > 0.8, "Mirrored top-right should be Red");
    assert!(
        out_mirrored[5] < 0.2,
        "Mirrored top-right should have low Blue"
    );
}

#[test]
fn test_preprocessor_bounds_and_error_handling() {
    let preprocessor = Preprocessor::new(MOVENET_INPUT_WIDTH, MOVENET_INPUT_HEIGHT);
    assert_eq!(preprocessor.target_len(), MOVENET_INPUT_LEN);

    let mut out_tensor = vec![0.0f32; MOVENET_INPUT_LEN];

    // Invalid source buffer length
    let truncated_src = vec![0u8; 100];
    let err = preprocessor.process_rgb8(&truncated_src, 640, 480, true, &mut out_tensor);
    assert!(err.is_err(), "Expected error on truncated source bytes");

    // Invalid output tensor length
    let valid_src = vec![0u8; 640 * 480 * 3];
    let mut short_out = vec![0.0f32; 10];
    let err2 = preprocessor.process_rgb8(&valid_src, 640, 480, true, &mut short_out);
    assert!(err2.is_err(), "Expected error on mismatched output tensor");

    // Zero dimensions
    let err3 = preprocessor.process_rgb8(&valid_src, 0, 480, true, &mut out_tensor);
    assert!(err3.is_err(), "Expected error on 0 width");

    // HWC to CHW conversion test
    let hwc = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]; // 2 pixels, 3 channels
    let small_prep = Preprocessor::new(2, 1);
    let mut chw = vec![0.0f32; 6];
    assert!(small_prep.hwc_to_chw(&hwc, &mut chw).is_ok());
    // Channel 0 (R): [1.0, 4.0], Channel 1 (G): [2.0, 5.0], Channel 2 (B): [3.0, 6.0]
    assert_eq!(chw, vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
}

#[test]
fn test_movenet_output_tensor_parser() {
    let mut raw = vec![0.0f32; MOVENET_OUTPUT_LEN];

    // Populate landmarks with [y, x, score] ordering:
    // Nose (COCO 0): y = 0.20, x = 0.50, conf = 0.95
    raw[COCO_NOSE * 3] = 0.20;
    raw[COCO_NOSE * 3 + 1] = 0.50;
    raw[COCO_NOSE * 3 + 2] = 0.95;

    // Left wrist (COCO 9): y = 0.40, x = 0.30, conf = 0.85 (below nose)
    raw[COCO_LEFT_WRIST * 3] = 0.40;
    raw[COCO_LEFT_WRIST * 3 + 1] = 0.30;
    raw[COCO_LEFT_WRIST * 3 + 2] = 0.85;

    // Right wrist (COCO 10): y = 0.10, x = 0.70, conf = 0.92 (raised above nose)
    raw[COCO_RIGHT_WRIST * 3] = 0.10;
    raw[COCO_RIGHT_WRIST * 3 + 1] = 0.70;
    raw[COCO_RIGHT_WRIST * 3 + 2] = 0.92;

    // Left ankle (COCO 15): low confidence below threshold
    raw[COCO_LEFT_ANKLE * 3] = 0.90;
    raw[COCO_LEFT_ANKLE * 3 + 1] = 0.45;
    raw[COCO_LEFT_ANKLE * 3 + 2] = 0.15; // below 0.25 threshold

    let skeleton = parse_movenet_output(&raw, 2, 0.25).expect("Parser should succeed");
    assert_eq!(skeleton.id, 2);

    // Coordinate verification: parser maps [y, x, conf] to Keypoint { x, y, confidence }
    assert!((skeleton.nose().x - 0.50).abs() < 1e-5);
    assert!((skeleton.nose().y - 0.20).abs() < 1e-5);
    assert!((skeleton.nose().confidence - 0.95).abs() < 1e-5);

    // Hand raised logic: right hand (y=0.10) is above nose (y=0.20)
    assert!(skeleton.is_hand_raised(true), "Right hand should be raised");
    // Left hand (y=0.40) is below nose (y=0.20)
    assert!(
        !skeleton.is_hand_raised(false),
        "Left hand should not be raised"
    );

    // Confidence thresholding: left ankle score was 0.15 < 0.25 threshold, so clamped to 0.0
    assert_eq!(skeleton.left_ankle().confidence, 0.0);
    assert!(!skeleton.left_ankle().is_confident(0.25));

    // Raw parser preserves the 0.15 score
    let raw_skel = parse_movenet_output_raw(&raw, 2).expect("Raw parser should succeed");
    assert!((raw_skel.left_ankle().confidence - 0.15).abs() < 1e-5);

    // Too short tensor returns error
    let short_tensor = vec![0.0f32; 10];
    assert!(parse_movenet_output(&short_tensor, 1, 0.25).is_err());
}

#[test]
fn test_adapter_mock_fallback_and_polling() {
    let config = CameraInferenceConfig {
        fps: 60,
        ..Default::default()
    };

    // In headless testing, physical camera is absent; with_mock_fallback must spawn cleanly without panic
    let mut adapter = CameraPoseStreamAdapter::with_mock_fallback(config, None);

    // Wait a brief tick for the worker thread to deliver the first frame
    std::thread::sleep(Duration::from_millis(40));

    let polled = adapter.poll_input().expect("poll_input must succeed");
    assert!(polled.is_some(), "Expected a polled synthetic frame");

    let input = polled.expect("frame present");
    assert!(
        !input.players.is_empty(),
        "Should contain 1 synthetic player"
    );

    let player = &input.players[0];
    assert_eq!(player.id, 1);
    assert!(player.nose().is_confident(0.5));
    assert!(player.nose().x >= 0.0 && player.nose().x <= 1.0);
    assert!(player.nose().y >= 0.0 && player.nose().y <= 1.0);
}

#[test]
fn test_adapter_headless_camera_open_fails_gracefully() {
    let config = CameraInferenceConfig {
        camera_index: 999, // Non-existent camera
        model_path: PathBuf::from("non_existent_model.onnx"),
        ..Default::default()
    };

    // CameraPoseStreamAdapter::new must return Err, never panic
    let result = CameraPoseStreamAdapter::new(config);
    assert!(
        result.is_err(),
        "Expected CaptureFailure or InferenceFailure on invalid hardware/model"
    );
}

#[test]
fn test_zero_panic_guarantee_no_unwrap_in_production_code() {
    let src_files = [
        "src/lib.rs",
        "src/config.rs",
        "src/preprocess.rs",
        "src/movenet.rs",
        "src/adapter.rs",
    ];

    for file_rel in &src_files {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = PathBuf::from(manifest_dir).join(file_rel);
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Could not read file {:?}: {}", path, e));

        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                continue;
            }
            if trimmed.contains(".unwrap()") {
                panic!(
                    "Forbidden .unwrap() found in production code {}:{} -> {}",
                    file_rel,
                    line_idx + 1,
                    trimmed
                );
            }
            if trimmed.contains(".expect(") {
                panic!(
                    "Forbidden .expect() found in production code {}:{} -> {}",
                    file_rel,
                    line_idx + 1,
                    trimmed
                );
            }
        }
    }
}
