use core::game::Game;
use core::pose::{Keypoint, PlayerInput, Skeleton, COCO_RIGHT_WRIST, KEYPOINT_COUNT};
use deflecto_bot::{DeflectoBot, Quadrant, PARTICLES_PER_EXPLOSION};
use learning::bridge::{LearningSessionPort, LeitnerLearningSession};
use learning::pack::ContentPack;
use std::sync::{Arc, Mutex};

/// Helper to load the bundled math pack from repository path.
fn load_bundled_pack() -> ContentPack {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let candidates = [
        format!(
            "{}/../../../content/bundled/math/number_facts_k2.toml",
            manifest_dir
        ),
        format!(
            "{}/../../content/bundled/math/number_facts_k2.toml",
            manifest_dir
        ),
        "content/bundled/math/number_facts_k2.toml".to_string(),
    ];
    for c in &candidates {
        if std::path::Path::new(c).exists() {
            return ContentPack::from_file(c).expect("Failed to parse bundled pack");
        }
    }
    panic!(
        "Could not find number_facts_k2.toml in candidates: {:?}",
        candidates
    );
}

/// Helper to construct player input with wrist precisely positioned at a quadrant center.
fn make_wrist_input(quadrant: Quadrant) -> PlayerInput {
    let (wx, wy) = quadrant.center();
    let mut keypoints = [Keypoint::ZERO; KEYPOINT_COUNT];
    keypoints[COCO_RIGHT_WRIST] = Keypoint::new(wx, wy, 0.95);
    let skel = Skeleton::new(1, keypoints);
    PlayerInput::new(vec![skel], 1.0)
}

/// Helper to advance the simulation by a given duration with empty input.
fn advance_simulation(bot: &mut DeflectoBot, duration_secs: f32) {
    let dt = 1.0 / 60.0;
    let steps = (duration_secs / dt).ceil() as usize;
    let empty_input = PlayerInput::empty(0.0);
    for _ in 0..steps {
        bot.update(dt, &empty_input);
    }
}

#[test]
fn test_deflecto_bot_headless_simulation_answers_5_questions_with_leitner_progression() {
    // 1. Load the official bundled K-2 math facts pack
    let pack = load_bundled_pack();
    assert_eq!(pack.items.len(), 10);

    // 2. Wrap Leitner session in Arc<Mutex<...>> for inspecting internal Leitner state
    let session = Arc::new(Mutex::new(LeitnerLearningSession::new(pack)));
    let mut bot = DeflectoBot::new_learning(Box::new(Arc::clone(&session)));
    bot.set_spawn_interval(0.5); // fast interval for deterministic headless simulation
    bot.init();

    assert!(bot.is_learning_mode());
    assert_eq!(bot.score(), 0);

    // Initial state: all 10 items in Box 1
    assert_eq!(
        session.lock().unwrap().scheduler().box_counts(),
        [10, 0, 0, 0, 0]
    );

    // 3. Question 1: Answering correctly
    assert!(bot.current_prompt().is_some());
    assert_eq!(bot.orbs().len(), 4);

    let correct_orb = bot
        .orbs()
        .iter()
        .find(|o| o.is_correct)
        .cloned()
        .expect("Wave must contain 1 correct orb");

    let input1 = make_wrist_input(correct_orb.quadrant);
    bot.update(0.016, &input1);

    // Verified: Score increments, gold particles spawn, orbs cleared
    assert_eq!(bot.score(), 1);
    assert_eq!(bot.orbs().len(), 0);
    assert_eq!(bot.particles().len(), PARTICLES_PER_EXPLOSION);
    assert!(bot.punch_intensity() > 0.9);

    // Advance to Question 2
    advance_simulation(&mut bot, 0.6);
    assert_eq!(bot.orbs().len(), 4);

    // 4. Question 2: Incorrect deflection followed by correct recovery!
    let wrong_orb = bot
        .orbs()
        .iter()
        .find(|o| !o.is_correct)
        .cloned()
        .expect("Wave must contain distractors");
    let right_orb = bot
        .orbs()
        .iter()
        .find(|o| o.is_correct)
        .cloned()
        .expect("Wave must contain 1 correct orb");

    // Hit the wrong orb
    let wrong_input = make_wrist_input(wrong_orb.quadrant);
    bot.update(0.016, &wrong_input);

    // Score did NOT increment, soft red dissolve spawned, correct orb still alive!
    assert_eq!(bot.score(), 1);
    assert!(bot.punch_intensity() <= 0.35);
    assert!(bot.orbs().iter().any(|o| o.is_correct));

    // Now deflect the correct orb
    let right_input = make_wrist_input(right_orb.quadrant);
    bot.update(0.016, &right_input);

    // Score increments to 2, orbs cleared
    assert_eq!(bot.score(), 2);
    assert_eq!(bot.orbs().len(), 0);

    // Advance to Question 3
    advance_simulation(&mut bot, 0.6);

    // 5. Questions 3, 4, 5: Answer correctly
    for expected_score in 3..=5 {
        assert_eq!(bot.orbs().len(), 4, "Question wave must have 4 orbs");
        let target_orb = bot
            .orbs()
            .iter()
            .find(|o| o.is_correct)
            .cloned()
            .expect("Must have correct orb");

        let input = make_wrist_input(target_orb.quadrant);
        bot.update(0.016, &input);

        assert_eq!(bot.score(), expected_score);
        assert_eq!(bot.orbs().len(), 0);

        if expected_score < 5 {
            advance_simulation(&mut bot, 0.6);
        }
    }

    // 6. Assert Leitner progression and session statistics
    assert_eq!(bot.score(), 5);

    let session_guard = session.lock().unwrap();
    let (correct, total) = session_guard.session_stats();
    assert_eq!(correct, 5);
    assert_eq!(total, 6); // 5 correct + 1 wrong attempt

    let scheduler = session_guard.scheduler();
    assert_eq!(scheduler.total_correct(), 5);
    assert_eq!(scheduler.total_attempts(), 6);
    assert!(scheduler.overall_accuracy() > 0.80);
}

#[test]
fn test_leitner_box_2_promotion_via_gameplay_deflections() {
    let pack = load_bundled_pack();

    let session = Arc::new(Mutex::new(LeitnerLearningSession::new(pack)));
    let mut bot = DeflectoBot::new_learning(Box::new(Arc::clone(&session)));
    bot.set_spawn_interval(0.2);
    bot.init();

    // Answer questions until an item is promoted to Box 2
    // (requires 2 consecutive correct answers on the same item)
    for _ in 0..20 {
        if bot.orbs().is_empty() {
            advance_simulation(&mut bot, 0.25);
        }

        if let Some(correct_orb) = bot.orbs().iter().find(|o| o.is_correct).cloned() {
            let input = make_wrist_input(correct_orb.quadrant);
            bot.update(0.016, &input);
        }

        let box_counts = session.lock().unwrap().scheduler().box_counts();
        if box_counts[1] > 0 {
            // At least one item successfully promoted to Box 2!
            break;
        }
    }

    let box_counts = session.lock().unwrap().scheduler().box_counts();
    assert!(
        box_counts[1] > 0,
        "Expected at least 1 item promoted to Box 2 after repeated correct deflections, got: {:?}",
        box_counts
    );
}

#[test]
fn test_deflecto_bot_60_fps_simulation_continuous_running() {
    let pack = load_bundled_pack();

    let session = Box::new(LeitnerLearningSession::new(pack));
    let mut bot = DeflectoBot::new_learning(session);
    bot.init();

    let dt = 1.0 / 60.0;
    // Run 300 simulation frames (5 seconds) with synthetic hovering hands
    for frame in 0..300 {
        let t = (frame as f32) * dt;
        // Hovering right hand that periodically punches TopLeft quadrant
        let wx = 0.25 + 0.1 * (t * 2.0).sin();
        let wy = 0.25 + 0.1 * (t * 2.0).cos();

        let mut keypoints = [Keypoint::ZERO; KEYPOINT_COUNT];
        keypoints[COCO_RIGHT_WRIST] = Keypoint::new(wx, wy, 0.9);
        let skel = Skeleton::new(1, keypoints);
        let input = PlayerInput::new(vec![skel], t as f64);

        let status = bot.update(dt, &input);
        assert_eq!(status, core::game::GameStatus::Running);
    }
}
