use app::{generate_synthetic_wave, run_simulation_step, PoseSource, SKELETON_BONES};
use core::game::{CanvasPort, Game, GameStatus, RenderContext};
use core::pose::{COCO_LEFT_WRIST, COCO_RIGHT_WRIST};
use deflecto_bot::{DeflectoBot, Quadrant, PARTICLES_PER_EXPLOSION};

#[derive(Default)]
#[allow(dead_code)]
struct TestLine {
    start: (f32, f32),
    end: (f32, f32),
    thickness: f32,
}

#[derive(Default)]
struct TestCanvas {
    circles: Vec<(f32, f32, f32)>,
    rings: Vec<(f32, f32, f32, f32)>,
    texts: Vec<String>,
    lines: Vec<TestLine>,
}

impl CanvasPort for TestCanvas {
    fn draw_circle(&mut self, x: f32, y: f32, radius: f32, _color: [f32; 4]) {
        self.circles.push((x, y, radius));
    }

    fn draw_ring(&mut self, x: f32, y: f32, radius: f32, thickness: f32, _color: [f32; 4]) {
        self.rings.push((x, y, radius, thickness));
    }

    fn draw_text(&mut self, text: &str, _x: f32, _y: f32, _font_size: f32, _color: [f32; 4]) {
        self.texts.push(text.to_string());
    }

    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, _color: [f32; 4]) {
        self.lines.push(TestLine {
            start: (x1, y1),
            end: (x2, y2),
            thickness,
        });
    }
}

#[test]
fn test_deflecto_bot_60_fps_simulation_loop_with_sample_wave() {
    let sample_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/recordings/sample_wave.jsonl"
    );
    let mut source =
        PoseSource::from_mock_path(sample_path).expect("Failed to load sample_wave.jsonl");

    let mut game = DeflectoBot::new();
    game.init();

    let dt = 1.0 / 60.0;
    let mut total_score = 0;

    // Simulate 120 frames (2.0 seconds of gameplay at 60 FPS)
    for _ in 0..120 {
        let status =
            run_simulation_step(&mut game, &mut source, dt).expect("simulation tick failed");
        assert_eq!(status, GameStatus::Running);
        total_score = game.score();
    }

    // Verify game was active and progressing
    assert!(game.punch_intensity() >= 0.0);
    println!("Simulation finished with score: {}", total_score);

    // Render check with mock canvas
    let mut canvas = TestCanvas::default();
    let mut ctx = RenderContext::new(1280.0, 720.0, dt, &mut canvas);
    game.draw(&mut ctx);

    // Verify drawing calls were issued
    assert!(!canvas.lines.is_empty(), "Quadrant lines should be drawn");
    assert!(!canvas.rings.is_empty(), "Target rings should be drawn");
    assert!(
        !canvas.texts.is_empty(),
        "Score and HUD text should be drawn"
    );
    assert!(
        canvas.texts.iter().any(|t| t.contains("SCORE")),
        "HUD must display score"
    );
}

#[test]
fn test_procedural_synthetic_player_wave_reaches_quadrants() {
    let mut hit_top_left = false;
    let mut hit_top_right = false;

    // Sample across 3 seconds of synthetic wave
    for i in 0..180 {
        let t = (i as f64) / 60.0;
        let input = generate_synthetic_wave(t);
        let skel = &input.players[0];

        let lw = &skel.keypoints[COCO_LEFT_WRIST];
        let rw = &skel.keypoints[COCO_RIGHT_WRIST];

        if Quadrant::from_point(lw.x, lw.y) == Some(Quadrant::TopLeft) {
            hit_top_left = true;
        }
        if Quadrant::from_point(rw.x, rw.y) == Some(Quadrant::TopRight) {
            hit_top_right = true;
        }
    }

    assert!(
        hit_top_left,
        "Synthetic wave must reach into TopLeft quadrant"
    );
    assert!(
        hit_top_right,
        "Synthetic wave must reach into TopRight quadrant"
    );
}

#[test]
fn test_deflecto_bot_exact_particle_explosion() {
    let mut game = DeflectoBot::new();
    game.orbs.clear();
    let orb_id = game.spawn_orb(Quadrant::TopLeft, "ORB", true);
    assert_eq!(game.orbs.len(), 1);
    assert_eq!(game.particles.len(), 0);

    // Advance orb near strike line
    game.orbs[0].progress = 0.8;

    // Feed player input with wrist in TopLeft
    let mut input = generate_synthetic_wave(0.0);
    // Explicitly place right wrist in TopLeft quadrant
    input.players[0].keypoints[COCO_RIGHT_WRIST].x = 0.2;
    input.players[0].keypoints[COCO_RIGHT_WRIST].y = 0.2;
    input.players[0].keypoints[COCO_RIGHT_WRIST].confidence = 0.95;

    let status = game.update(0.016, &input);
    assert_eq!(status, GameStatus::Running);

    // Check that orb was deflected
    assert!(game.orbs.iter().all(|o| o.id != orb_id));
    assert_eq!(game.score, 1);
    // Exact 30 particles requirement!
    assert_eq!(game.particles.len(), PARTICLES_PER_EXPLOSION);
    assert_eq!(game.particles.len(), 30);
    // Punch effect triggered
    assert!(game.punch_intensity > 0.8);
}

#[test]
fn test_skeleton_bone_connections_completeness() {
    // 19 standard COCO connections
    assert_eq!(SKELETON_BONES.len(), 19);
    for &(k1, k2) in &SKELETON_BONES {
        assert!(k1 < 17);
        assert!(k2 < 17);
        assert_ne!(k1, k2);
    }
}

#[test]
fn test_camera_pose_stream_adapter_headless_fallback_integration() {
    use app::CameraInferenceConfig;

    let config = CameraInferenceConfig {
        camera_index: 0,
        fps: 60,
        ..Default::default()
    };

    // In headless environment without webcam or ONNX file, must fallback cleanly without panic
    let mut source = PoseSource::from_camera_config(config, None);
    std::thread::sleep(std::time::Duration::from_millis(40));

    let mut game = DeflectoBot::new();
    game.init();

    let dt = 1.0 / 60.0;
    for _ in 0..60 {
        let status =
            run_simulation_step(&mut game, &mut source, dt).expect("simulation step failed");
        assert_eq!(status, GameStatus::Running);
    }
}

#[test]
fn test_camera_pose_stream_adapter_with_recording_fallback() {
    use app::CameraInferenceConfig;
    use core::pose::COCO_NOSE;
    use std::path::PathBuf;

    let sample_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/recordings/sample_wave.jsonl"
    );
    let config = CameraInferenceConfig {
        camera_index: 99,
        fps: 60,
        ..Default::default()
    };

    let mut source = PoseSource::from_camera_config(config, Some(PathBuf::from(sample_path)));
    std::thread::sleep(std::time::Duration::from_millis(40));

    let frame = source.poll_input().expect("poll_input must succeed");
    assert!(!frame.players.is_empty(), "Expected player from recording");
    assert!(frame.players[0].keypoints[COCO_NOSE].is_confident(0.5));
}

#[test]
fn test_camera_pose_source_poll_frame_interface() {
    use app::CameraInferenceConfig;

    let config = CameraInferenceConfig {
        fps: 60,
        ..Default::default()
    };
    let mut source = PoseSource::from_camera_config(config, None);
    std::thread::sleep(std::time::Duration::from_millis(40));

    // poll_frame returns Option<PlayerInput>
    let frame_opt = source.poll_frame().expect("poll_frame must succeed");
    assert!(frame_opt.is_some());

    // poll_input returns PlayerInput
    let frame_direct = source.poll_input().expect("poll_input must succeed");
    assert_eq!(frame_direct.players.len(), 1);
}

#[test]
fn test_zero_panic_guarantee_no_unwrap_in_app_production_code() {
    use std::path::PathBuf;

    let src_files = ["src/lib.rs", "src/main.rs", "src/menu.rs"];

    for file_rel in &src_files {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = PathBuf::from(manifest_dir).join(file_rel);
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Could not read file {:?}: {}", path, e));

        let mut in_tests = false;
        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("#[cfg(test)]") || trimmed.starts_with("mod tests") {
                in_tests = true;
            }
            if in_tests {
                continue;
            }
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
