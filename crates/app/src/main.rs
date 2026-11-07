use app::{draw_floor_ring, draw_neon_player_silhouette, MacroquadCanvas, PoseSource};
use clap::Parser;
use core::game::{Game, RenderContext};
use core::pose::PlayerInput;
use deflecto_bot::DeflectoBot;
use macroquad::prelude::*;
use std::path::PathBuf;

/// Command line arguments for MotionArcade native runner.
#[derive(Parser, Debug)]
#[command(
    name = "motion-arcade",
    about = "MotionArcade native game loop and keypoint visualizer"
)]
struct CliArgs {
    /// Path to pre-recorded JSONL pose stream file for deterministic playback
    #[arg(long)]
    mock_input: Option<PathBuf>,

    /// Run for N simulation frames and cleanly exit (ideal for headless CI verification)
    #[arg(long)]
    test_frames: Option<u32>,

    /// Window width in pixels
    #[arg(long, default_value_t = 1280)]
    width: i32,

    /// Window height in pixels
    #[arg(long, default_value_t = 720)]
    height: i32,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "MotionArcade - Deflecto-Bot Native Runner".to_string(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let _ = tracing_subscriber::fmt::try_init();

    let args = CliArgs::parse();

    let mut pose_source = match &args.mock_input {
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
    };

    let mut game = DeflectoBot::new();
    game.init();

    let mut canvas = MacroquadCanvas;
    let fixed_dt = 1.0 / 60.0;
    let mut accumulator = 0.0f32;
    let mut total_time = 0.0f32;
    let mut frame_count: u32 = 0;
    let mut latest_input = PlayerInput::empty(0.0);

    tracing::info!("Entering locked 60 FPS native game loop");

    loop {
        let frame_time = get_frame_time().min(0.1);
        accumulator += frame_time;

        // Locked 60 FPS fixed timestep update
        while accumulator >= fixed_dt {
            match pose_source.poll_input() {
                Ok(input) => {
                    latest_input = input;
                }
                Err(e) => {
                    tracing::warn!("Pose input poll error: {}", e);
                }
            }

            game.update(fixed_dt, &latest_input);

            accumulator -= fixed_dt;
            total_time += fixed_dt;
            frame_count = frame_count.wrapping_add(1);

            // Handle headless test exit condition
            if let Some(target_frames) = args.test_frames {
                if frame_count >= target_frames {
                    tracing::info!(
                        "Completed {} test frames cleanly. Final score: {}",
                        frame_count,
                        game.score()
                    );
                    return;
                }
            }
        }

        // Render pass
        let sw = screen_width();
        let sh = screen_height();

        // Cyber arena background
        clear_background(Color::new(0.02, 0.03, 0.06, 1.0));

        // 1. Draw glowing floor ring (anchor boundary cylinder)
        let floor_cx = sw * 0.5;
        let floor_cy = sh * 0.84;
        let floor_rx = sw * 0.28;
        let floor_ry = sh * 0.11;
        draw_floor_ring(floor_cx, floor_cy, floor_rx, floor_ry, total_time);

        // 2. Draw neon glowing player silhouette mirroring keypoints
        for player in &latest_input.players {
            draw_neon_player_silhouette(player, sw, sh, total_time);
        }

        // 3. Draw Deflecto-Bot game, orbs, and explosion particles
        let mut ctx = RenderContext::new(sw, sh, fixed_dt, &mut canvas);
        game.draw(&mut ctx);

        next_frame().await;
    }
}
