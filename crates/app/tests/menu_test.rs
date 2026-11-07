use app::menu::{
    both_arms_up, check_exit_gesture, is_exit_gesture, is_t_pose, GestureMenu, SelectedGame,
    DEFAULT_PORTAL_RADIUS, LEFT_PORTAL_CENTER, RIGHT_PORTAL_CENTER,
};
use core::game::{CanvasPort, RenderContext};
use core::pose::{
    Keypoint, PlayerInput, Skeleton, COCO_LEFT_SHOULDER, COCO_LEFT_WRIST, COCO_NOSE,
    COCO_RIGHT_SHOULDER, COCO_RIGHT_WRIST, KEYPOINT_COUNT,
};

/// Helper to create a single-player input with specific left and right wrist positions.
fn make_wrist_input(
    lw: Option<(f32, f32, f32)>,
    rw: Option<(f32, f32, f32)>,
) -> PlayerInput {
    let mut keypoints = [Keypoint::ZERO; KEYPOINT_COUNT];

    // Standard head and torso anchor
    keypoints[COCO_NOSE] = Keypoint::new(0.5, 0.25, 0.95);
    keypoints[COCO_LEFT_SHOULDER] = Keypoint::new(0.42, 0.35, 0.95);
    keypoints[COCO_RIGHT_SHOULDER] = Keypoint::new(0.58, 0.35, 0.95);

    if let Some((x, y, conf)) = lw {
        keypoints[COCO_LEFT_WRIST] = Keypoint::new(x, y, conf);
    }
    if let Some((x, y, conf)) = rw {
        keypoints[COCO_RIGHT_WRIST] = Keypoint::new(x, y, conf);
    }

    let skel = Skeleton::new(1, keypoints);
    PlayerInput::new(vec![skel], 1.0)
}

#[derive(Default)]
struct MockCanvas {
    circles: Vec<(f32, f32, f32)>,
    rings: Vec<(f32, f32, f32, f32)>,
    texts: Vec<String>,
    lines_count: usize,
}

impl CanvasPort for MockCanvas {
    fn draw_circle(&mut self, x: f32, y: f32, radius: f32, _color: [f32; 4]) {
        self.circles.push((x, y, radius));
    }

    fn draw_ring(&mut self, x: f32, y: f32, radius: f32, thickness: f32, _color: [f32; 4]) {
        self.rings.push((x, y, radius, thickness));
    }

    fn draw_text(&mut self, text: &str, _x: f32, _y: f32, _font_size: f32, _color: [f32; 4]) {
        self.texts.push(text.to_string());
    }

    fn draw_line(
        &mut self,
        _x1: f32,
        _y1: f32,
        _x2: f32,
        _y2: f32,
        _thickness: f32,
        _color: [f32; 4],
    ) {
        self.lines_count += 1;
    }
}

#[test]
fn test_gesture_menu_initial_state() {
    let mut menu = GestureMenu::new();
    assert_eq!(menu.dwell_left_banana, 0.0);
    assert_eq!(menu.dwell_right_deflecto, 0.0);
    assert_eq!(menu.portal_radius, DEFAULT_PORTAL_RADIUS);
    assert_eq!(menu.is_selected(), SelectedGame::None);

    let empty = PlayerInput::empty(0.0);
    let selected = menu.update(0.016, &empty);
    assert_eq!(selected, SelectedGame::None);
}

#[test]
fn test_left_portal_dwell_selection_banana_grab() {
    let mut menu = GestureMenu::new();
    let dt = 1.0 / 60.0;

    // Place left wrist directly inside Left Portal (Banana Grab)
    let input = make_wrist_input(
        Some((LEFT_PORTAL_CENTER.0, LEFT_PORTAL_CENTER.1, 0.95)),
        None,
    );

    // After 30 frames (0.5s), dwell should be ~0.50
    for _ in 0..30 {
        let selected = menu.update(dt, &input);
        assert_eq!(selected, SelectedGame::None);
    }
    assert!(
        menu.dwell_left_banana >= 0.48 && menu.dwell_left_banana <= 0.52,
        "Dwell after 0.5s should be ~0.50, got {}",
        menu.dwell_left_banana
    );
    assert_eq!(menu.dwell_right_deflecto, 0.0);

    // After 30 more frames (total 60 frames = 1.0s), dwell reaches 1.0 and launches BananaGrab
    let mut launched = false;
    for _ in 30..60 {
        let selected = menu.update(dt, &input);
        if selected == SelectedGame::BananaGrab {
            launched = true;
        }
    }

    assert!(launched, "BananaGrab must be launched when dwell reaches 1.0s");
    assert_eq!(menu.dwell_left_banana, 1.0);
    assert_eq!(menu.is_selected(), SelectedGame::BananaGrab);
}

#[test]
fn test_right_portal_dwell_selection_deflecto_bot() {
    let mut menu = GestureMenu::new();
    let dt = 1.0 / 60.0;

    // Place right wrist directly inside Right Portal (Deflecto-Bot)
    let input = make_wrist_input(
        None,
        Some((RIGHT_PORTAL_CENTER.0, RIGHT_PORTAL_CENTER.1, 0.95)),
    );

    // After 30 frames (0.5s), dwell should be ~0.50
    for _ in 0..30 {
        let selected = menu.update(dt, &input);
        assert_eq!(selected, SelectedGame::None);
    }
    assert!(
        menu.dwell_right_deflecto >= 0.48 && menu.dwell_right_deflecto <= 0.52,
        "Dwell after 0.5s should be ~0.50, got {}",
        menu.dwell_right_deflecto
    );
    assert_eq!(menu.dwell_left_banana, 0.0);

    // After 30 more frames (total 60 frames = 1.0s), dwell reaches 1.0 and launches DeflectoBot
    let mut launched = false;
    for _ in 30..60 {
        let selected = menu.update(dt, &input);
        if selected == SelectedGame::DeflectoBot {
            launched = true;
        }
    }

    assert!(launched, "DeflectoBot must be launched when dwell reaches 1.0s");
    assert_eq!(menu.dwell_right_deflecto, 1.0);
    assert_eq!(menu.is_selected(), SelectedGame::DeflectoBot);
}

#[test]
fn test_dwell_smooth_reset_when_hand_leaves() {
    let mut menu = GestureMenu::new();
    let dt = 1.0 / 60.0;

    // Hand inside for 30 frames (~0.5s)
    let input = make_wrist_input(
        Some((LEFT_PORTAL_CENTER.0, LEFT_PORTAL_CENTER.1, 0.95)),
        None,
    );
    for _ in 0..30 {
        menu.update(dt, &input);
    }
    let midway_dwell = menu.dwell_left_banana;
    assert!(midway_dwell >= 0.45);

    // Hand leaves portal (empty input)
    let empty = PlayerInput::empty(0.0);
    menu.update(dt, &empty);
    assert!(
        menu.dwell_left_banana < midway_dwell,
        "Dwell must decrease when hand leaves portal"
    );

    // After decaying for 20 frames, dwell should smoothly approach 0.0
    for _ in 0..20 {
        menu.update(dt, &empty);
    }
    assert_eq!(menu.dwell_left_banana, 0.0);
}

#[test]
fn test_low_confidence_wrist_ignored() {
    let mut menu = GestureMenu::new();
    let dt = 1.0 / 60.0;

    // Low confidence wrist (0.10 < MIN_KEYPOINT_CONFIDENCE)
    let input = make_wrist_input(
        Some((LEFT_PORTAL_CENTER.0, LEFT_PORTAL_CENTER.1, 0.10)),
        None,
    );

    for _ in 0..30 {
        menu.update(dt, &input);
    }
    assert_eq!(menu.dwell_left_banana, 0.0);
}

#[test]
fn test_exit_gesture_both_arms_up() {
    // Both wrists raised high above shoulders and nose
    let input_up = make_wrist_input(
        Some((0.35, 0.15, 0.95)), // Above shoulder (0.35) and nose (0.25)
        Some((0.65, 0.15, 0.95)),
    );
    let skel_up = &input_up.players[0];
    assert!(both_arms_up(skel_up));
    assert!(is_exit_gesture(skel_up));
    assert!(check_exit_gesture(&input_up));

    // Arms down at sides
    let input_down = make_wrist_input(
        Some((0.38, 0.65, 0.95)),
        Some((0.62, 0.65, 0.95)),
    );
    let skel_down = &input_down.players[0];
    assert!(!both_arms_up(skel_down));
    assert!(!is_exit_gesture(skel_down));
    assert!(!check_exit_gesture(&input_down));

    // Only one arm up
    let input_one_up = make_wrist_input(
        Some((0.35, 0.15, 0.95)),
        Some((0.62, 0.65, 0.95)),
    );
    let skel_one_up = &input_one_up.players[0];
    assert!(!both_arms_up(skel_one_up));
    assert!(!is_exit_gesture(skel_one_up));
}

#[test]
fn test_exit_gesture_t_pose() {
    // T-pose: wide horizontal span at shoulder height (~0.35)
    let input_t = make_wrist_input(
        Some((0.15, 0.35, 0.95)), // far left, shoulder height
        Some((0.85, 0.35, 0.95)), // far right, shoulder height
    );
    let skel_t = &input_t.players[0];
    assert!(is_t_pose(skel_t));
    assert!(is_exit_gesture(skel_t));
    assert!(check_exit_gesture(&input_t));

    // Arms close together (not a T-pose)
    let input_close = make_wrist_input(
        Some((0.45, 0.35, 0.95)),
        Some((0.55, 0.35, 0.95)),
    );
    let skel_close = &input_close.players[0];
    assert!(!is_t_pose(skel_close));
}

#[test]
fn test_menu_draw_with_mock_canvas() {
    let mut menu = GestureMenu::new();
    let mut canvas = MockCanvas::default();
    let mut ctx = RenderContext::new(1280.0, 720.0, 0.016, &mut canvas);

    menu.draw(&mut ctx);

    // Verify drawing calls
    assert!(!canvas.rings.is_empty(), "Portal rings should be drawn");
    assert!(!canvas.circles.is_empty(), "Portal cores should be drawn");
    assert!(!canvas.texts.is_empty(), "HUD texts should be drawn");
    assert!(
        canvas.texts.iter().any(|t| t.contains("BANANA GRAB")),
        "Left portal must have Banana Grab text"
    );
    assert!(
        canvas.texts.iter().any(|t| t.contains("DEFLECTO-BOT")),
        "Right portal must have Deflecto-Bot text"
    );

    // Verify dwell radial fill rendering when dwell is active
    menu.dwell_left_banana = 0.5;
    menu.dwell_right_deflecto = 0.75;
    let mut active_canvas = MockCanvas::default();
    let mut active_ctx = RenderContext::new(1280.0, 720.0, 0.016, &mut active_canvas);
    menu.draw(&mut active_ctx);

    assert!(
        active_canvas.lines_count > 0,
        "Radial arc lines should be drawn for active dwell"
    );
    assert!(
        active_canvas.texts.iter().any(|t| t.contains("50%")),
        "Left dwell percentage should be drawn"
    );
    assert!(
        active_canvas.texts.iter().any(|t| t.contains("75%")),
        "Right dwell percentage should be drawn"
    );
}

#[test]
fn test_gesture_menu_reset() {
    let mut menu = GestureMenu::new();
    menu.dwell_left_banana = 1.0;
    menu.dwell_right_deflecto = 1.0;
    assert_eq!(menu.is_selected(), SelectedGame::BananaGrab);

    menu.reset();
    assert_eq!(menu.dwell_left_banana, 0.0);
    assert_eq!(menu.dwell_right_deflecto, 0.0);
    assert_eq!(menu.is_selected(), SelectedGame::None);
}
