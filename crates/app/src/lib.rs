use core::game::{CanvasPort, Game, GameStatus};
use core::ports::input::PoseStreamPort;
use core::ports::mock::MockFilePoseStreamAdapter;
use core::pose::{
    Keypoint, PlayerInput, Skeleton, COCO_LEFT_ANKLE, COCO_LEFT_EAR, COCO_LEFT_ELBOW,
    COCO_LEFT_EYE, COCO_LEFT_HIP, COCO_LEFT_KNEE, COCO_LEFT_SHOULDER, COCO_LEFT_WRIST, COCO_NOSE,
    COCO_RIGHT_ANKLE, COCO_RIGHT_EAR, COCO_RIGHT_ELBOW, COCO_RIGHT_EYE, COCO_RIGHT_HIP,
    COCO_RIGHT_KNEE, COCO_RIGHT_SHOULDER, COCO_RIGHT_WRIST, KEYPOINT_COUNT,
};
use macroquad::prelude::*;
use std::f32::consts::TAU;
use std::path::Path;

pub mod menu;
pub use menu::{
    both_arms_up, check_exit_gesture, is_exit_gesture, is_t_pose, GestureMenu, SelectedGame,
    DEFAULT_PORTAL_RADIUS, DWELL_TARGET_SECONDS, EXIT_GESTURE_DWELL_SECONDS, LEFT_PORTAL_CENTER,
    MIN_KEYPOINT_CONFIDENCE, RIGHT_PORTAL_CENTER,
};

/// Standard COCO skeletal bone connections as (start_index, end_index) tuples.
pub const SKELETON_BONES: [(usize, usize); 19] = [
    // Head facial landmarks
    (COCO_NOSE, COCO_LEFT_EYE),
    (COCO_NOSE, COCO_RIGHT_EYE),
    (COCO_LEFT_EYE, COCO_LEFT_EAR),
    (COCO_RIGHT_EYE, COCO_RIGHT_EAR),
    // Upper body / arms
    (COCO_LEFT_SHOULDER, COCO_LEFT_ELBOW),
    (COCO_LEFT_ELBOW, COCO_LEFT_WRIST),
    (COCO_RIGHT_SHOULDER, COCO_RIGHT_ELBOW),
    (COCO_RIGHT_ELBOW, COCO_RIGHT_WRIST),
    // Torso box
    (COCO_LEFT_SHOULDER, COCO_RIGHT_SHOULDER),
    (COCO_LEFT_SHOULDER, COCO_LEFT_HIP),
    (COCO_RIGHT_SHOULDER, COCO_RIGHT_HIP),
    (COCO_LEFT_HIP, COCO_RIGHT_HIP),
    // Lower body / legs
    (COCO_LEFT_HIP, COCO_LEFT_KNEE),
    (COCO_LEFT_KNEE, COCO_LEFT_ANKLE),
    (COCO_RIGHT_HIP, COCO_RIGHT_KNEE),
    (COCO_RIGHT_KNEE, COCO_RIGHT_ANKLE),
    // Neck midpoint to nose anchor
    (COCO_LEFT_SHOULDER, COCO_NOSE),
    (COCO_RIGHT_SHOULDER, COCO_NOSE),
    // Pelvis center connector
    (COCO_LEFT_HIP, COCO_RIGHT_HIP),
];

/// Macroquad graphics implementation of the core domain `CanvasPort`.
#[derive(Debug, Default)]
pub struct MacroquadCanvas;

impl CanvasPort for MacroquadCanvas {
    fn draw_circle(&mut self, x: f32, y: f32, radius: f32, color: [f32; 4]) {
        draw_circle(
            x,
            y,
            radius,
            Color::new(color[0], color[1], color[2], color[3]),
        );
    }

    fn draw_ring(&mut self, x: f32, y: f32, radius: f32, thickness: f32, color: [f32; 4]) {
        draw_poly_lines(
            x,
            y,
            40,
            radius,
            0.0,
            thickness,
            Color::new(color[0], color[1], color[2], color[3]),
        );
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: [f32; 4]) {
        draw_text(
            text,
            x,
            y,
            font_size,
            Color::new(color[0], color[1], color[2], color[3]),
        );
    }

    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: [f32; 4]) {
        draw_line(
            x1,
            y1,
            x2,
            y2,
            thickness,
            Color::new(color[0], color[1], color[2], color[3]),
        );
    }
}

/// Generates a procedural synthetic player wave payload for simulation without a camera or mock file.
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

    // Left arm: dynamic wave reaching across TopLeft quadrant
    let lw_x = 0.26 + (t * 2.2).sin() * 0.15;
    let lw_y = 0.30 - (t * 1.8).cos() * 0.18;
    let le_x = (ls_x + lw_x) * 0.5 - 0.03;
    let le_y = (ls_y + lw_y) * 0.5;
    keypoints[COCO_LEFT_ELBOW] = Keypoint::new(le_x, le_y, 0.94);
    keypoints[COCO_LEFT_WRIST] = Keypoint::new(lw_x, lw_y, 0.96);

    // Right arm: dynamic wave reaching across TopRight quadrant
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
    keypoints[COCO_RIGHT_HIP] = Keypoint::new(rh_x, rh_y, 0.95);

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

/// Renders a multi-layer neon glowing player silhouette with mirrored keypoints.
pub fn draw_neon_player_silhouette(
    skeleton: &Skeleton,
    screen_width: f32,
    screen_height: f32,
    pulse_t: f32,
) {
    let min_conf = 0.25;

    // Helper: Mirrors x coordinate horizontally to create mirror effect for the player
    let to_screen = |kp: &Keypoint| -> (f32, f32) {
        let mirrored_x = (1.0 - kp.x) * screen_width;
        let y = kp.y * screen_height;
        (mirrored_x, y)
    };

    // 1. Draw glowing bone segments
    for &(start_idx, end_idx) in &SKELETON_BONES {
        let k1 = &skeleton.keypoints[start_idx];
        let k2 = &skeleton.keypoints[end_idx];

        if k1.is_confident(min_conf) && k2.is_confident(min_conf) {
            let (x1, y1) = to_screen(k1);
            let (x2, y2) = to_screen(k2);

            // Layer 1: Outer glow aura (wide, transparent cyan)
            draw_line(x1, y1, x2, y2, 16.0, Color::new(0.0, 0.85, 1.0, 0.16));

            // Layer 2: Mid neon highlight
            draw_line(x1, y1, x2, y2, 7.5, Color::new(0.2, 0.95, 1.0, 0.45));

            // Layer 3: Sharp core laser line
            draw_line(x1, y1, x2, y2, 3.0, Color::new(0.9, 0.98, 1.0, 0.95));
        }
    }

    // 2. Draw glowing joint nodes
    for (idx, kp) in skeleton.keypoints.iter().enumerate() {
        if kp.is_confident(min_conf) {
            let (sx, sy) = to_screen(kp);

            // Special handling for wrists (Deflector shield hitzones)
            if idx == COCO_LEFT_WRIST || idx == COCO_RIGHT_WRIST {
                let pulse = (pulse_t * 6.5).sin() * 3.5;
                let shield_r = 26.0 + pulse;

                // Pulsing outer deflection aura
                draw_poly_lines(
                    sx,
                    sy,
                    32,
                    shield_r,
                    0.0,
                    3.5,
                    Color::new(1.0, 0.88, 0.2, 0.85),
                );
                draw_circle(sx, sy, shield_r * 0.7, Color::new(1.0, 0.92, 0.35, 0.25));
                draw_circle(sx, sy, 7.0, Color::new(1.0, 1.0, 1.0, 0.95));
            } else {
                // Standard joint node
                draw_circle(sx, sy, 12.0, Color::new(0.1, 0.85, 1.0, 0.22));
                draw_circle(sx, sy, 5.5, Color::new(0.85, 0.98, 1.0, 0.9));
            }
        }
    }
}

/// Renders a glowing floor cylinder boundary ring with perspective foreshortening.
pub fn draw_floor_ring(center_x: f32, center_y: f32, radius_x: f32, radius_y: f32, pulse_t: f32) {
    let pulse = (pulse_t * 2.8).sin() * 0.15;
    let segments = 48;

    let draw_ellipse = |rx: f32, ry: f32, thickness: f32, color: Color| {
        let step = TAU / (segments as f32);
        for i in 0..segments {
            let a1 = (i as f32) * step;
            let a2 = ((i + 1) as f32) * step;
            let x1 = center_x + a1.cos() * rx;
            let y1 = center_y + a1.sin() * ry;
            let x2 = center_x + a2.cos() * rx;
            let y2 = center_y + a2.sin() * ry;
            draw_line(x1, y1, x2, y2, thickness, color);
        }
    };

    // Outer glow ring
    draw_ellipse(
        radius_x * (1.0 + pulse * 0.05),
        radius_y * (1.0 + pulse * 0.05),
        4.0,
        Color::new(0.05, 0.75, 1.0, 0.35 + pulse * 0.1),
    );

    // Mid sharp ring
    draw_ellipse(
        radius_x * 0.92,
        radius_y * 0.92,
        2.5,
        Color::new(0.2, 0.95, 0.85, 0.75 + pulse * 0.1),
    );

    // Inner subtle ring
    draw_ellipse(
        radius_x * 0.82,
        radius_y * 0.82,
        1.5,
        Color::new(0.1, 0.7, 0.9, 0.25),
    );
}

/// Inbound stream source that provides either a looping mock recording or real-time synthetic waves.
pub enum PoseSource {
    Mock(MockFilePoseStreamAdapter),
    Synthetic { start_instant: std::time::Instant },
}

impl PoseSource {
    pub fn from_mock_path<P: AsRef<Path>>(path: P) -> Result<Self, core::TrackingError> {
        let adapter = MockFilePoseStreamAdapter::from_file(path)?.with_looping(true);
        Ok(PoseSource::Mock(adapter))
    }

    pub fn synthetic() -> Self {
        PoseSource::Synthetic {
            start_instant: std::time::Instant::now(),
        }
    }

    pub fn poll_input(&mut self) -> Result<PlayerInput, core::TrackingError> {
        match self {
            PoseSource::Mock(adapter) => match adapter.poll_input()? {
                Some(input) => Ok(input),
                None => Ok(PlayerInput::empty(0.0)),
            },
            PoseSource::Synthetic { start_instant } => {
                let elapsed = start_instant.elapsed().as_secs_f64();
                Ok(generate_synthetic_wave(elapsed))
            }
        }
    }
}

/// Executes a single fixed simulation tick of the app.
pub fn run_simulation_step(
    game: &mut dyn Game,
    source: &mut PoseSource,
    dt: f32,
) -> Result<GameStatus, core::TrackingError> {
    let input = source.poll_input()?;
    let status = game.update(dt, &input);
    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use deflecto_bot::{DeflectoBot, Quadrant};

    #[test]
    fn test_synthetic_wave_generation() {
        let input_0 = generate_synthetic_wave(0.0);
        let input_1 = generate_synthetic_wave(1.0);

        assert_eq!(input_0.players.len(), 1);
        assert_eq!(input_1.players.len(), 1);

        let skel0 = &input_0.players[0];
        let skel1 = &input_1.players[0];

        // Should have 17 valid keypoints
        assert_eq!(skel0.keypoints.len(), 17);
        assert!(skel0.keypoints[COCO_LEFT_WRIST].confidence >= 0.9);
        assert!(skel0.keypoints[COCO_RIGHT_WRIST].confidence >= 0.9);

        // Arm coordinates should change smoothly over time
        let dx = (skel0.keypoints[COCO_LEFT_WRIST].x - skel1.keypoints[COCO_LEFT_WRIST].x).abs();
        let dy = (skel0.keypoints[COCO_LEFT_WRIST].y - skel1.keypoints[COCO_LEFT_WRIST].y).abs();
        assert!(dx + dy > 0.01);
    }

    #[test]
    fn test_mock_stream_playback_with_sample_wave_asset() {
        let sample_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/recordings/sample_wave.jsonl"
        );
        let mut source =
            PoseSource::from_mock_path(sample_path).expect("Failed to load sample_wave.jsonl");

        for _ in 0..150 {
            let input = source.poll_input().expect("poll failed");
            assert_eq!(input.players.len(), 1);
            assert!(input.players[0].keypoints[COCO_NOSE].is_confident(0.5));
        }
    }

    #[test]
    fn test_simulation_step_updates_deflecto_bot() {
        let mut bot = DeflectoBot::new();
        bot.orbs.clear();
        bot.spawn_orb(Quadrant::TopLeft, "test", true);

        let mut source = PoseSource::synthetic();
        let status =
            run_simulation_step(&mut bot, &mut source, 0.016).expect("simulation step failed");

        assert_eq!(status, GameStatus::Running);
    }
}
