use core::boundary::FloorBoundary;
use core::ports::{MockFilePoseStreamAdapter, PoseStreamPort};
use core::pose::{Keypoint, PlayerInput, Skeleton, KEYPOINT_COUNT};
use core::smoothing::{OneEuroFilter, OneEuroFilterConfig, SkeletonSmoother};

// Deterministic linear congruential generator for reproducible test noise
struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_f32(&mut self) -> f32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let val = ((self.state >> 33) & 0x7FFFFFFF) as f32;
        (val / 0x7FFFFFFF as f32) * 2.0 - 1.0 // Normalized between -1.0 and 1.0
    }
}

#[test]
fn test_one_euro_filter_noisy_sine_wave_reduces_jitter_over_60_percent() {
    let config = OneEuroFilterConfig {
        min_cutoff: 0.8,
        beta: 0.005,
        d_cutoff: 1.0,
    };
    let mut filter = OneEuroFilter::new(config);
    let mut rng = DeterministicRng::new(12345);

    let fps = 60.0;
    let dt = 1.0 / fps;
    let duration_seconds = 5.0;
    let total_steps = (duration_seconds * fps) as usize;

    let signal_freq = 0.5; // 0.5 Hz smooth wave (slow hand movement)
    let noise_amplitude = 0.08; // High frequency landmark jitter

    let mut noisy_step_diff = 0.0f32;
    let mut filtered_step_diff = 0.0f32;

    let mut prev_noisy = 0.0f32;
    let mut prev_filtered = 0.0f32;

    for step in 0..total_steps {
        let t = step as f64 * dt;
        let clean = (2.0 * std::f32::consts::PI * signal_freq * t as f32).sin() * 0.3 + 0.5;
        let noise = rng.next_f32() * noise_amplitude;
        let noisy = clean + noise;

        let filtered = filter.filter(noisy, t);

        // Allow initial filter burn-in (first 20 frames) before accumulating metrics
        if step > 20 {
            noisy_step_diff += (noisy - prev_noisy).abs();
            filtered_step_diff += (filtered - prev_filtered).abs();
        }

        prev_noisy = noisy;
        prev_filtered = filtered;
    }

    let jitter_reduction = 1.0 - (filtered_step_diff / noisy_step_diff);
    println!(
        "Noisy step diff: {:.4}, Filtered step diff: {:.4}, Jitter Reduction: {:.2}%",
        noisy_step_diff,
        filtered_step_diff,
        jitter_reduction * 100.0
    );

    // Acceptance criteria: jitter reduction must strictly exceed 60%
    assert!(
        jitter_reduction > 0.60,
        "Expected jitter reduction > 60%, achieved {:.2}%",
        jitter_reduction * 100.0
    );
}

#[test]
fn test_one_euro_filter_adaptive_fast_motion() {
    let config = OneEuroFilterConfig {
        min_cutoff: 1.0,
        beta: 0.02,
        d_cutoff: 1.0,
    };
    let mut filter = OneEuroFilter::new(config);

    // Steady state at 0.0
    for i in 0..30 {
        let t = i as f64 * 0.016;
        filter.filter(0.0, t);
    }

    // Sudden rapid jump from 0.0 to 1.0
    let jump_val = 1.0;
    let jump_time = 30.0 * 0.016;
    let first_jump = filter.filter(jump_val, jump_time);

    // Due to adaptive beta and velocity increase, the filter should respond quickly
    let next_jump = filter.filter(jump_val, jump_time + 0.016);
    assert!(
        next_jump > first_jump,
        "Filter should rapidly converge towards high speed input"
    );
}

#[test]
fn test_skeleton_smoother_integration() {
    let config = OneEuroFilterConfig::default();
    let mut smoother = SkeletonSmoother::new(config);

    let mut keypoints = [Keypoint::ZERO; KEYPOINT_COUNT];
    for kp in &mut keypoints {
        *kp = Keypoint::new(0.5, 0.5, 0.9);
    }
    let raw_skeleton = Skeleton::new(1, keypoints);

    let smoothed_1 = smoother.filter_skeleton(&raw_skeleton, 0.0);
    assert_eq!(smoothed_1.id, 1);
    assert!((smoothed_1.nose().x - 0.5).abs() < 1e-4);

    // Move wrist with jitter
    let mut noisy_keypoints = keypoints;
    noisy_keypoints[9] = Keypoint::new(0.58, 0.52, 0.9);
    let noisy_skeleton = Skeleton::new(1, noisy_keypoints);

    let smoothed_2 = smoother.filter_skeleton(&noisy_skeleton, 0.033);
    assert!(smoothed_2.left_wrist().x > 0.50 && smoothed_2.left_wrist().x < 0.58);
}

#[test]
fn test_full_pipeline_mock_stream_filtering_and_boundary() {
    let boundary = FloorBoundary::new(0.5, 0.5, 0.35);
    let mut smoother = SkeletonSmoother::new(OneEuroFilterConfig::default());

    // Generate mock JSON lines
    let mut lines = Vec::new();
    for i in 0..10 {
        let t = i as f64 * 0.033;
        let mut kp_inside = [Keypoint::ZERO; KEYPOINT_COUNT];
        // Root inside boundary at (0.5, 0.5)
        kp_inside[11] = Keypoint::new(0.48, 0.5, 0.9);
        kp_inside[12] = Keypoint::new(0.52, 0.5, 0.9);
        let player_inside = Skeleton::new(1, kp_inside);

        let mut kp_outside = [Keypoint::ZERO; KEYPOINT_COUNT];
        // Root outside boundary at (0.95, 0.95)
        kp_outside[11] = Keypoint::new(0.94, 0.95, 0.9);
        kp_outside[12] = Keypoint::new(0.96, 0.95, 0.9);
        let player_outside = Skeleton::new(2, kp_outside);

        let frame = PlayerInput::new(vec![player_inside, player_outside], t);
        let json_line = serde_json::to_string(&frame).expect("serialize frame");
        lines.push(json_line);
    }

    let mut adapter = MockFilePoseStreamAdapter::new(lines);

    let mut processed_frames = 0;
    while let Ok(Some(input)) = adapter.poll_input() {
        // Step 1: Filter spatial boundary
        let bounded_input = boundary.filter_player_input(input);

        // Phantom player 2 must be discarded
        assert_eq!(bounded_input.players.len(), 1);
        assert_eq!(bounded_input.players[0].id, 1);

        // Step 2: Smooth active skeletons
        let smoothed_player =
            smoother.filter_skeleton(&bounded_input.players[0], bounded_input.timestamp);
        assert_eq!(smoothed_player.id, 1);

        processed_frames += 1;
    }

    assert_eq!(processed_frames, 10);
}
