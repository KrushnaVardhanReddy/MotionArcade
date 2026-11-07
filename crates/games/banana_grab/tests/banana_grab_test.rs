use banana_grab::{
    hip_height_ratio, is_ducking_pose, is_near_fruit, is_wrist_raised, BananaGrab, FallingFruit,
    DEFAULT_FRUIT_SPEED, DUCK_RATIO_THRESHOLD, MIN_WRIST_CONFIDENCE, PARTICLES_PER_COLLECTION,
};
use core::game::{CanvasPort, Game, GameStatus, RenderContext};
use core::pose::{
    Keypoint, PlayerInput, Skeleton, COCO_LEFT_ANKLE, COCO_LEFT_HIP, COCO_LEFT_SHOULDER,
    COCO_LEFT_WRIST, COCO_NOSE, COCO_RIGHT_ANKLE, COCO_RIGHT_HIP, COCO_RIGHT_SHOULDER,
    COCO_RIGHT_WRIST, KEYPOINT_COUNT,
};

/// Mock canvas implementation to verify 2D rendering commands.
struct MockCanvas {
    pub circle_count: usize,
    pub ring_count: usize,
    pub line_count: usize,
    pub text_count: usize,
}

impl MockCanvas {
    fn new() -> Self {
        Self {
            circle_count: 0,
            ring_count: 0,
            line_count: 0,
            text_count: 0,
        }
    }
}

impl CanvasPort for MockCanvas {
    fn draw_circle(&mut self, _x: f32, _y: f32, _radius: f32, _color: [f32; 4]) {
        self.circle_count += 1;
    }

    fn draw_ring(&mut self, _x: f32, _y: f32, _radius: f32, _thickness: f32, _color: [f32; 4]) {
        self.ring_count += 1;
    }

    fn draw_text(&mut self, _text: &str, _x: f32, _y: f32, _font_size: f32, _color: [f32; 4]) {
        self.text_count += 1;
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
        self.line_count += 1;
    }
}

/// Helper: Builds a standard standing skeleton centered at `x`.
fn standing_skeleton(id: u8, x: f32) -> Skeleton {
    let mut keypoints = [Keypoint::ZERO; KEYPOINT_COUNT];
    keypoints[COCO_NOSE] = Keypoint::new(x, 0.20, 0.95);
    keypoints[COCO_LEFT_SHOULDER] = Keypoint::new(x - 0.08, 0.30, 0.90);
    keypoints[COCO_RIGHT_SHOULDER] = Keypoint::new(x + 0.08, 0.30, 0.90);
    keypoints[COCO_LEFT_WRIST] = Keypoint::new(x - 0.12, 0.45, 0.90);
    keypoints[COCO_RIGHT_WRIST] = Keypoint::new(x + 0.12, 0.45, 0.90);
    keypoints[COCO_LEFT_HIP] = Keypoint::new(x - 0.06, 0.60, 0.90);
    keypoints[COCO_RIGHT_HIP] = Keypoint::new(x + 0.06, 0.60, 0.90);
    keypoints[COCO_LEFT_ANKLE] = Keypoint::new(x - 0.06, 0.95, 0.90);
    keypoints[COCO_RIGHT_ANKLE] = Keypoint::new(x + 0.06, 0.95, 0.90);

    Skeleton::new(id, keypoints)
}

/// Helper: Builds a ducking / squatting skeleton centered at `x`.
fn ducking_skeleton(id: u8, x: f32) -> Skeleton {
    let mut keypoints = [Keypoint::ZERO; KEYPOINT_COUNT];
    // Head and torso descend towards the ground
    keypoints[COCO_NOSE] = Keypoint::new(x, 0.50, 0.95);
    keypoints[COCO_LEFT_SHOULDER] = Keypoint::new(x - 0.08, 0.60, 0.90);
    keypoints[COCO_RIGHT_SHOULDER] = Keypoint::new(x + 0.08, 0.60, 0.90);
    keypoints[COCO_LEFT_WRIST] = Keypoint::new(x - 0.12, 0.68, 0.90);
    keypoints[COCO_RIGHT_WRIST] = Keypoint::new(x + 0.12, 0.68, 0.90);
    // Hips drop down close to ankles
    keypoints[COCO_LEFT_HIP] = Keypoint::new(x - 0.06, 0.85, 0.90);
    keypoints[COCO_RIGHT_HIP] = Keypoint::new(x + 0.06, 0.85, 0.90);
    keypoints[COCO_LEFT_ANKLE] = Keypoint::new(x - 0.06, 0.95, 0.90);
    keypoints[COCO_RIGHT_ANKLE] = Keypoint::new(x + 0.06, 0.95, 0.90);

    Skeleton::new(id, keypoints)
}

/// Helper: Builds a skeleton reaching high with one hand.
fn high_reach_skeleton(id: u8, x: f32, is_right: bool, hand_x: f32, hand_y: f32) -> Skeleton {
    let mut skel = standing_skeleton(id, x);
    let kp = Keypoint::new(hand_x, hand_y, 0.95);
    if is_right {
        skel.keypoints[COCO_RIGHT_WRIST] = kp;
    } else {
        skel.keypoints[COCO_LEFT_WRIST] = kp;
    }
    skel
}

#[test]
fn test_game_trait_lifecycle() {
    let mut game = BananaGrab::new();

    // Verify contract identifiers
    assert_eq!(game.id(), "banana_grab");
    assert_eq!(game.name(), "Jungle Jump: Banana Grab");

    // Mutate state
    game.score = 42;
    game.bananas_collected = 10;
    game.fruits.clear();
    game.particles.clear();

    // Reset via init()
    game.init();
    assert_eq!(game.score, 0);
    assert_eq!(game.bananas_collected, 0);
    assert!(!game.fruits.is_empty());
    assert!(game.particles.is_empty());
    assert_eq!(game.status, GameStatus::Running);

    // Update tick
    let empty_input = PlayerInput::empty(0.0);
    let status = game.update(0.016, &empty_input);
    assert_eq!(status, GameStatus::Running);

    // Draw tick
    let mut canvas = MockCanvas::new();
    let mut ctx = RenderContext::new(800.0, 600.0, 0.016, &mut canvas);
    game.draw(&mut ctx);

    // Verify all 2D primitive categories were exercised
    assert!(canvas.line_count > 0);
    assert!(canvas.circle_count > 0);
    assert!(canvas.ring_count > 0);
    assert!(canvas.text_count > 0);
}

#[test]
fn test_fruit_falling_physics() {
    let mut game = BananaGrab::new();
    game.fruits.clear();

    let speed = 0.50;
    let initial_y = 0.10;
    game.spawn_fruit(0.5, initial_y, speed, "🍌", true);
    assert_eq!(game.fruits.len(), 1);

    // Advance by 0.2 seconds
    let dt = 0.20;
    let empty_input = PlayerInput::empty(0.0);
    game.update(dt, &empty_input);

    let expected_y = initial_y + speed * dt;
    assert!((game.fruits[0].y - expected_y).abs() < 1e-4);

    // Advance past screen bottom (y > 1.15)
    game.update(3.0, &empty_input);
    // Fruit should be removed as it fell off-screen
    assert!(game.fruits.iter().all(|f| f.y <= 1.15));
}

#[test]
fn test_high_reach_fruit_grab_right_hand() {
    let mut game = BananaGrab::new();
    game.fruits.clear();
    game.particles.clear();

    // Spawn fruit at (0.5, 0.18)
    game.spawn_fruit(0.5, 0.18, 0.0, "🍌", true);
    assert_eq!(game.fruits.len(), 1);
    assert_eq!(game.bananas_collected, 0);
    assert_eq!(game.score, 0);

    // Player with right wrist raised above nose (y = 0.16 < nose y 0.20)
    let skel = high_reach_skeleton(1, 0.5, true, 0.50, 0.16);
    let input = PlayerInput::new(vec![skel], 1.0);

    let status = game.update(0.016, &input);
    assert_eq!(status, GameStatus::Running);

    // Fruit grabbed!
    assert_eq!(game.fruits.len(), 0);
    assert_eq!(game.bananas_collected, 1);
    assert_eq!(game.score, 1);

    // Exactly 25 tropical leaf particles spawned!
    assert_eq!(game.particles.len(), PARTICLES_PER_COLLECTION);
    assert_eq!(game.particles.len(), 25);

    // Verify green leaf colors
    for particle in &game.particles {
        assert!(
            particle.color[1] > 0.6,
            "Particle green channel must be high"
        );
        assert_eq!(particle.color[3], 1.0, "Alpha should start at 1.0");
    }
}

#[test]
fn test_high_reach_fruit_grab_left_hand() {
    let mut game = BananaGrab::new();
    game.fruits.clear();
    game.particles.clear();

    // Spawn fruit at (0.35, 0.17)
    game.spawn_fruit(0.35, 0.17, 0.0, "🥭", true);

    // Player centered at 0.40 reaching with left hand to (0.35, 0.15)
    let skel = high_reach_skeleton(1, 0.40, false, 0.35, 0.15);
    let input = PlayerInput::new(vec![skel], 1.0);

    game.update(0.016, &input);

    assert_eq!(game.bananas_collected, 1);
    assert_eq!(game.score, 1);
    assert_eq!(game.particles.len(), 25);
    assert_eq!(game.fruits.len(), 0);
}

#[test]
fn test_low_hand_does_not_grab_fruit() {
    let mut game = BananaGrab::new();
    game.fruits.clear();
    game.particles.clear();

    // Fruit at lower height (0.5, 0.65)
    game.spawn_fruit(0.5, 0.65, 0.0, "🍌", true);

    // Player with hand down at hip height (wrist y = 0.65, but nose y = 0.20)
    // Hand is NOT raised!
    let mut skel = standing_skeleton(1, 0.5);
    skel.keypoints[COCO_RIGHT_WRIST] = Keypoint::new(0.5, 0.65, 0.95);
    let input = PlayerInput::new(vec![skel], 1.0);

    game.update(0.016, &input);

    // Fruit should NOT be collected because child did not reach up
    assert_eq!(game.fruits.len(), 1);
    assert_eq!(game.bananas_collected, 0);
    assert_eq!(game.score, 0);
    assert!(game.particles.is_empty());
}

#[test]
fn test_low_confidence_wrist_ignored() {
    let mut game = BananaGrab::new();
    game.fruits.clear();
    game.particles.clear();

    game.spawn_fruit(0.5, 0.18, 0.0, "🍌", true);

    // Wrist in position but confidence below MIN_WRIST_CONFIDENCE threshold
    let mut skel = standing_skeleton(1, 0.5);
    skel.keypoints[COCO_RIGHT_WRIST] = Keypoint::new(0.5, 0.16, MIN_WRIST_CONFIDENCE - 0.05);
    let input = PlayerInput::new(vec![skel], 1.0);

    game.update(0.016, &input);

    assert_eq!(game.fruits.len(), 1);
    assert_eq!(game.bananas_collected, 0);
    assert!(game.particles.is_empty());
}

#[test]
fn test_particle_generation_and_lifetime_decay() {
    let mut game = BananaGrab::new();
    game.particles.clear();

    // Trigger burst directly
    game.spawn_leaf_burst(0.5, 0.5);
    assert_eq!(game.particles.len(), 25);

    // Verify initial state
    for p in &game.particles {
        assert_eq!(p.lifetime, 0.0);
        assert!(p.max_lifetime >= 0.5);
        assert!(p.is_alive());
    }

    // Step simulation by 0.1 seconds
    let empty_input = PlayerInput::empty(0.0);
    game.update(0.1, &empty_input);

    for p in &game.particles {
        assert!((p.lifetime - 0.1).abs() < 1e-4);
        assert!(p.is_alive());
    }

    // Advance beyond all max_lifetimes (>= 1.0s)
    game.update(1.2, &empty_input);
    assert!(
        game.particles.is_empty(),
        "All expired particles must be purged"
    );
}

#[test]
fn test_ducking_logic_hip_height_ratio() {
    let standing = standing_skeleton(1, 0.5);
    let ducking = ducking_skeleton(1, 0.5);

    let standing_ratio = hip_height_ratio(&standing);
    let ducking_ratio = hip_height_ratio(&ducking);

    // Standing tall ratio should be comfortably above threshold
    assert!(
        standing_ratio > DUCK_RATIO_THRESHOLD,
        "Standing ratio {standing_ratio} should exceed threshold {DUCK_RATIO_THRESHOLD}"
    );
    assert!(!is_ducking_pose(&standing));

    // Ducking / squatting drops the ratio significantly
    assert!(
        ducking_ratio < DUCK_RATIO_THRESHOLD,
        "Ducking ratio {ducking_ratio} must drop below threshold {DUCK_RATIO_THRESHOLD}"
    );
    assert!(is_ducking_pose(&ducking));
}

#[test]
fn test_vine_ducking_evasion() {
    let mut game = BananaGrab::new();
    // Position vine right over center
    game.swinging_vine_x = 0.50;
    game.vine_speed = 0.0; // keep stationary for deterministic test

    // 1. Standing player under the vine -> vine contact occurs, no dodge
    let standing = standing_skeleton(1, 0.50);
    let input_standing = PlayerInput::new(vec![standing], 1.0);

    game.update(0.016, &input_standing);
    assert!(!game.is_ducking);
    assert!(game.vine_contact);
    assert_eq!(game.vine_hits, 1);
    assert_eq!(game.vine_dodges, 0);

    // Reset contact flag
    game.vine_contact = false;

    // 2. Ducking player under the vine -> dodges successfully!
    let ducking = ducking_skeleton(1, 0.50);
    let input_ducking = PlayerInput::new(vec![ducking], 2.0);

    game.update(0.016, &input_ducking);
    assert!(game.is_ducking);
    assert!(game.vine_dodged);
    assert_eq!(game.vine_dodges, 1);
    assert!(game.score >= 2, "Bonus score awarded for dodging vine");
}

#[test]
fn test_vine_sweeping_oscillation() {
    let mut game = BananaGrab::new();
    game.swinging_vine_x = 0.90;
    game.vine_dir = 1.0;
    game.vine_speed = 1.0;

    let empty_input = PlayerInput::empty(0.0);

    // Advance to hit right boundary (0.95)
    game.update(0.1, &empty_input);
    assert_eq!(
        game.vine_dir, -1.0,
        "Vine must bounce when reaching right edge"
    );
    assert!(game.swinging_vine_x <= 0.95);

    // Advance across towards left
    game.update(0.95, &empty_input);
    assert_eq!(
        game.vine_dir, 1.0,
        "Vine must bounce when reaching left edge"
    );
    assert!(game.swinging_vine_x >= 0.05);
}

#[test]
fn test_procedural_fruit_spawning() {
    let mut game = BananaGrab::new();
    game.fruits.clear();
    game.spawn_timer = 0.0;
    game.spawn_interval = 0.5;

    let empty_input = PlayerInput::empty(0.0);

    // Tick past spawn interval
    game.update(0.6, &empty_input);
    assert!(
        !game.fruits.is_empty(),
        "Fruit must be procedurally spawned"
    );

    let fruit = &game.fruits[0];
    assert!(fruit.x >= 0.1 && fruit.x <= 0.9);
    assert_eq!(fruit.y, 0.0);
    assert!(fruit.is_target);

    // On next frame, it falls
    game.update(0.1, &empty_input);
    assert!(game.fruits[0].y > 0.0);
}

#[test]
fn test_helper_predicates() {
    let standing = standing_skeleton(1, 0.5);
    // Left wrist at (0.38, 0.45) is below nose at 0.20
    assert!(!is_wrist_raised(&standing, false));
    assert!(!is_wrist_raised(&standing, true));

    // Raised wrist
    let reaching = high_reach_skeleton(1, 0.5, true, 0.5, 0.15);
    assert!(is_wrist_raised(&reaching, true));
    assert!(!is_wrist_raised(&reaching, false));

    // Near fruit predicate
    let fruit = FallingFruit::new(1, 0.5, 0.20, DEFAULT_FRUIT_SPEED, "🍌", true);
    let kp_close = Keypoint::new(0.52, 0.22, 0.9);
    let kp_far = Keypoint::new(0.85, 0.22, 0.9);

    assert!(is_near_fruit(&kp_close, &fruit));
    assert!(!is_near_fruit(&kp_far, &fruit));
}
