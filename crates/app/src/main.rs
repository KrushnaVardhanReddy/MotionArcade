use app::{
    check_exit_gesture, draw_floor_ring, draw_neon_player_silhouette, CameraInferenceConfig,
    CameraPoseStreamAdapter, GestureMenu, MacroquadCanvas, PoseSource, SelectedGame,
    EXIT_GESTURE_DWELL_SECONDS,
};
use audio::{AudioMixer, SoundEffect};
use banana_grab::BananaGrab;
use clap::Parser;
use core::game::{CanvasPort, Game, RenderContext};
use core::pose::PlayerInput;
use deflecto_bot::DeflectoBot;
use macroquad::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

/// Command line arguments for MotionArcade native runner.
#[derive(Parser, Debug)]
#[command(
    name = "motion-arcade",
    about = "MotionArcade native game loop and hands-free gesture portal runner"
)]
struct CliArgs {
    /// Path to pre-recorded JSONL pose stream file for deterministic playback
    #[arg(long)]
    mock_input: Option<PathBuf>,

    /// Run for N simulation frames and cleanly exit (ideal for headless CI verification)
    #[arg(long)]
    test_frames: Option<u32>,

    /// Path to TOML content pack file (defaults to content/bundled/math/number_facts_k2.toml)
    #[arg(long)]
    pack: Option<PathBuf>,

    /// Force pure arcade mode even if content pack exists
    #[arg(long)]
    arcade: bool,

    /// Initial game mode: "menu" (default), "deflecto", or "banana"
    #[arg(long, default_value = "menu")]
    game: String,

    /// Window width in pixels
    #[arg(long, default_value_t = 1280)]
    width: i32,

    /// Window height in pixels
    #[arg(long, default_value_t = 720)]
    height: i32,

    /// Camera device index (defaults to 0 for primary webcam)
    #[arg(long, default_value_t = 0)]
    camera: u32,

    /// Disable physical camera capture and use synthetic wave or mock file
    #[arg(long)]
    no_camera: bool,

    /// Path to MoveNet ONNX model file
    #[arg(long, default_value = "assets/models/movenet_lightning.onnx")]
    model: PathBuf,

    /// Mirror camera feed horizontally (default: true)
    #[arg(long, default_value_t = true, num_args = 0..=1, default_missing_value = "true")]
    mirror: bool,
}

/// App mode state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppState {
    Menu,
    Deflecto,
    Banana,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "MotionArcade - Hands-Free Gesture Arcade".to_string(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}

/// Helper to render the exit gesture countdown progress bar at the top of the screen.
fn draw_exit_progress(
    elapsed: f32,
    target: f32,
    sw: f32,
    _sh: f32,
    canvas: &mut dyn CanvasPort,
) {
    let progress = (elapsed / target).clamp(0.0, 1.0);
    let bar_width = 320.0;
    let bar_height = 14.0;
    let bx = sw * 0.5 - bar_width * 0.5;
    let by = 35.0;

    canvas.draw_text(
        "RETURNING TO MENU (HOLD GESTURE)...",
        bx - 10.0,
        by - 12.0,
        20.0,
        [1.0, 0.9, 0.3, 0.95],
    );

    // Background bar
    canvas.draw_line(bx, by, bx + bar_width, by, bar_height, [0.2, 0.2, 0.3, 0.6]);
    // Fill progress
    if progress > 0.005 {
        canvas.draw_line(
            bx,
            by,
            bx + bar_width * progress,
            by,
            bar_height,
            [1.0, 0.85, 0.2, 0.95],
        );
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let _ = tracing_subscriber::fmt::try_init();

    let args = CliArgs::parse();

    let mut pose_source = if args.no_camera {
        tracing::info!("Camera disabled via --no-camera flag.");
        match &args.mock_input {
            Some(path) => match PoseSource::from_mock_path(path) {
                Ok(src) => {
                    tracing::info!("Loaded mock pose stream from {:?}", path);
                    src
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to load mock recording from {:?}: {}. Falling back to synthetic wave.",
                        path,
                        e
                    );
                    PoseSource::synthetic()
                }
            },
            None => {
                tracing::info!("Running procedural synthetic player wave.");
                PoseSource::synthetic()
            }
        }
    } else {
        // Auto-detect camera hardware and ONNX model file; fall back gracefully to mock or synthetic wave
        let model_path = if args.model.exists() {
            args.model.clone()
        } else {
            let candidates = [
                args.model.clone(),
                PathBuf::from("../../").join(&args.model),
                PathBuf::from("../").join(&args.model),
            ];
            let mut resolved = args.model.clone();
            for c in candidates {
                if c.exists() {
                    resolved = c;
                    break;
                }
            }
            resolved
        };

        let config = CameraInferenceConfig {
            camera_index: args.camera,
            capture_width: 640,
            capture_height: 480,
            fps: 30,
            model_path,
            min_confidence: 0.25,
            mirror_horizontal: args.mirror,
        };

        let fallback_mock = match &args.mock_input {
            Some(path) => Some(path.clone()),
            None => {
                let candidates = [
                    PathBuf::from("assets/recordings/sample_wave.jsonl"),
                    PathBuf::from("../../assets/recordings/sample_wave.jsonl"),
                    PathBuf::from("../assets/recordings/sample_wave.jsonl"),
                ];
                let mut found = None;
                for c in &candidates {
                    if c.exists() {
                        found = Some(c.clone());
                        break;
                    }
                }
                found
            }
        };

        tracing::info!(
            "Initializing CameraPoseStreamAdapter (camera_index: {}, mirror: {}, model: {:?})",
            config.camera_index,
            config.mirror_horizontal,
            config.model_path
        );

        let adapter = CameraPoseStreamAdapter::with_mock_fallback(config, fallback_mock);
        PoseSource::camera(adapter)
    };

    let audio_mixer = Arc::new(AudioMixer::new());

    let mut deflecto_game = if args.arcade {
        tracing::info!("Arcade mode explicitly requested via --arcade");
        DeflectoBot::new_arcade().with_audio(Arc::clone(&audio_mixer))
    } else {
        let pack_path = if let Some(p) = &args.pack {
            p.clone()
        } else {
            let candidates = [
                PathBuf::from("content/bundled/math/number_facts_k2.toml"),
                PathBuf::from("../../content/bundled/math/number_facts_k2.toml"),
                PathBuf::from("../content/bundled/math/number_facts_k2.toml"),
            ];
            let mut found = PathBuf::from("content/bundled/math/number_facts_k2.toml");
            for c in &candidates {
                if c.exists() {
                    found = c.clone();
                    break;
                }
            }
            found
        };

        if pack_path.exists() {
            match learning::bridge::LeitnerLearningSession::from_file(&pack_path) {
                Ok(session) => {
                    tracing::info!("Loaded educational content pack from {:?}", pack_path);
                    DeflectoBot::new_learning(Box::new(session)).with_audio(Arc::clone(&audio_mixer))
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse content pack from {:?}: {}. Falling back to arcade mode.",
                        pack_path,
                        e
                    );
                    DeflectoBot::new_arcade().with_audio(Arc::clone(&audio_mixer))
                }
            }
        } else {
            tracing::info!(
                "Content pack not found at {:?}. Falling back to arcade mode.",
                pack_path
            );
            DeflectoBot::new_arcade().with_audio(Arc::clone(&audio_mixer))
        }
    };
    deflecto_game.init();

    let mut banana_game = BananaGrab::new().with_audio(Arc::clone(&audio_mixer));
    banana_game.init();

    let mut menu = GestureMenu::new();

    let initial_state = match args.game.to_lowercase().as_str() {
        "deflecto" | "deflecto_bot" | "deflectobot" => AppState::Deflecto,
        "banana" | "banana_grab" | "bananagrab" => AppState::Banana,
        _ => AppState::Menu,
    };
    let mut state = initial_state;
    let mut exit_gesture_timer = 0.0f32;

    let mut canvas = MacroquadCanvas;
    let fixed_dt = 1.0 / 60.0;
    let mut accumulator = 0.0f32;
    let mut total_time = 0.0f32;
    let mut frame_count: u32 = 0;
    let mut latest_input = PlayerInput::empty(0.0);

    tracing::info!("Entering locked 60 FPS native game loop in state {:?}", state);

    loop {
        let frame_time = get_frame_time().min(0.1);
        accumulator += frame_time;

        // Locked 60 FPS fixed timestep update
        while accumulator >= fixed_dt {
            match pose_source.poll_frame() {
                Ok(Some(input)) => {
                    latest_input = input;
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!("Pose input poll error: {}", e);
                }
            }

            // Keyboard navigation helpers (Escape / M to menu, 1 for Banana, 2 for Deflecto)
            if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::M) {
                if state != AppState::Menu {
                    tracing::info!("Escape/M key pressed: returning to menu");
                    state = AppState::Menu;
                    menu.reset();
                    exit_gesture_timer = 0.0;
                }
            } else if is_key_pressed(KeyCode::Key1) {
                tracing::info!("Key 1 pressed: launching Banana Grab");
                state = AppState::Banana;
                banana_game.init();
                exit_gesture_timer = 0.0;
            } else if is_key_pressed(KeyCode::Key2) {
                tracing::info!("Key 2 pressed: launching Deflecto-Bot");
                state = AppState::Deflecto;
                deflecto_game.init();
                exit_gesture_timer = 0.0;
            }

            // State Machine Transition Logic
            match state {
                AppState::Menu => {
                    exit_gesture_timer = 0.0;
                    let selected = menu.update(fixed_dt, &latest_input);
                    match selected {
                        SelectedGame::BananaGrab => {
                            tracing::info!("Menu: Banana Grab selected via 1.0s dwell gesture!");
                            audio_mixer.play(SoundEffect::BananaChomp);
                            state = AppState::Banana;
                            banana_game.init();
                            menu.reset();
                        }
                        SelectedGame::DeflectoBot => {
                            tracing::info!("Menu: Deflecto-Bot selected via 1.0s dwell gesture!");
                            audio_mixer.play(SoundEffect::CorrectFanfare);
                            state = AppState::Deflecto;
                            deflecto_game.init();
                            menu.reset();
                        }
                        SelectedGame::None => {}
                    }
                }
                AppState::Deflecto => {
                    deflecto_game.update(fixed_dt, &latest_input);

                    // Check exit gesture (T-pose or both arms up for 3.0s)
                    if check_exit_gesture(&latest_input) {
                        exit_gesture_timer += fixed_dt;
                        if exit_gesture_timer >= EXIT_GESTURE_DWELL_SECONDS {
                            tracing::info!("Exit gesture sustained for 3.0s: returning to menu");
                            audio_mixer.play(SoundEffect::GentleWobble);
                            state = AppState::Menu;
                            menu.reset();
                            exit_gesture_timer = 0.0;
                        }
                    } else {
                        exit_gesture_timer = (exit_gesture_timer - fixed_dt * 2.0).max(0.0);
                    }
                }
                AppState::Banana => {
                    banana_game.update(fixed_dt, &latest_input);

                    // Check exit gesture (T-pose or both arms up for 3.0s)
                    if check_exit_gesture(&latest_input) {
                        exit_gesture_timer += fixed_dt;
                        if exit_gesture_timer >= EXIT_GESTURE_DWELL_SECONDS {
                            tracing::info!("Exit gesture sustained for 3.0s: returning to menu");
                            audio_mixer.play(SoundEffect::GentleWobble);
                            state = AppState::Menu;
                            menu.reset();
                            exit_gesture_timer = 0.0;
                        }
                    } else {
                        exit_gesture_timer = (exit_gesture_timer - fixed_dt * 2.0).max(0.0);
                    }
                }
            }

            accumulator -= fixed_dt;
            total_time += fixed_dt;
            frame_count = frame_count.wrapping_add(1);

            // Handle headless test exit condition
            if let Some(target_frames) = args.test_frames {
                if frame_count >= target_frames {
                    tracing::info!(
                        "Completed {} test frames cleanly in state {:?}",
                        frame_count,
                        state
                    );
                    return;
                }
            }
        }

        // Render pass
        let sw = screen_width();
        let sh = screen_height();

        let floor_cx = sw * 0.5;
        let floor_cy = sh * 0.84;
        let floor_rx = sw * 0.28;
        let floor_ry = sh * 0.11;

        match state {
            AppState::Menu => {
                // Cyber dark background
                clear_background(Color::new(0.02, 0.03, 0.06, 1.0));

                // 1. Draw glowing floor ring
                draw_floor_ring(floor_cx, floor_cy, floor_rx, floor_ry, total_time);

                // 2. Draw neon glowing player silhouette mirroring keypoints
                for player in &latest_input.players {
                    draw_neon_player_silhouette(player, sw, sh, total_time);
                }

                // 3. Draw Gesture Portal Menu
                let mut ctx = RenderContext::new(sw, sh, fixed_dt, &mut canvas);
                menu.draw(&mut ctx);
            }
            AppState::Deflecto => {
                // Cyber arena background
                clear_background(Color::new(0.02, 0.03, 0.06, 1.0));

                draw_floor_ring(floor_cx, floor_cy, floor_rx, floor_ry, total_time);

                for player in &latest_input.players {
                    draw_neon_player_silhouette(player, sw, sh, total_time);
                }

                let mut ctx = RenderContext::new(sw, sh, fixed_dt, &mut canvas);
                deflecto_game.draw(&mut ctx);

                if exit_gesture_timer > 0.0 {
                    draw_exit_progress(
                        exit_gesture_timer,
                        EXIT_GESTURE_DWELL_SECONDS,
                        sw,
                        sh,
                        &mut canvas,
                    );
                }
            }
            AppState::Banana => {
                // Tropical jungle background
                clear_background(Color::new(0.02, 0.05, 0.03, 1.0));

                draw_floor_ring(floor_cx, floor_cy, floor_rx, floor_ry, total_time);

                for player in &latest_input.players {
                    draw_neon_player_silhouette(player, sw, sh, total_time);
                }

                let mut ctx = RenderContext::new(sw, sh, fixed_dt, &mut canvas);
                banana_game.draw(&mut ctx);

                if exit_gesture_timer > 0.0 {
                    draw_exit_progress(
                        exit_gesture_timer,
                        EXIT_GESTURE_DWELL_SECONDS,
                        sw,
                        sh,
                        &mut canvas,
                    );
                }
            }
        }

        next_frame().await;
    }
}
