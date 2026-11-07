//! Hands-Free Gesture Portal Menu
//!
//! Provides an interactive portal menu for children:
//! - Left Portal (Jungle Theme): Jungle Banana Grab
//! - Right Portal (Cyber Theme): Deflecto-Bot Math
//! - 1.0s sustained dwell timer with 60 FPS radial fill animation
//! - Hands-free exit gesture detection (T-pose or both arms up for 3.0s)

use core::game::{CanvasPort, RenderContext};
use core::pose::{
    PlayerInput, Skeleton, COCO_LEFT_SHOULDER, COCO_LEFT_WRIST, COCO_NOSE,
    COCO_RIGHT_SHOULDER, COCO_RIGHT_WRIST,
};
use std::f32::consts::{FRAC_PI_2, TAU};

/// Center coordinates for the left portal (Banana Grab) in normalized space [0.0, 1.0].
pub const LEFT_PORTAL_CENTER: (f32, f32) = (0.25, 0.45);

/// Center coordinates for the right portal (Deflecto-Bot) in normalized space [0.0, 1.0].
pub const RIGHT_PORTAL_CENTER: (f32, f32) = (0.75, 0.45);

/// Default normalized radius for portal interaction zones.
pub const DEFAULT_PORTAL_RADIUS: f32 = 0.18;

/// Target dwell duration in seconds required to trigger game selection.
pub const DWELL_TARGET_SECONDS: f32 = 1.0;

/// Target dwell duration in seconds required for the exit gesture.
pub const EXIT_GESTURE_DWELL_SECONDS: f32 = 3.0;

/// Minimum confidence threshold for keypoint detection.
pub const MIN_KEYPOINT_CONFIDENCE: f32 = 0.20;

/// Game selection outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedGame {
    None,
    DeflectoBot,
    BananaGrab,
}

/// Hands-free gesture portal menu state machine.
#[derive(Debug, Clone, PartialEq)]
pub struct GestureMenu {
    pub dwell_left_banana: f32, // 0.0 to 1.0 (1.0 = launch)
    pub dwell_right_deflecto: f32,
    pub portal_radius: f32,
}

impl Default for GestureMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureMenu {
    /// Creates a new `GestureMenu` with initial dwell timers at 0.0.
    pub fn new() -> Self {
        Self {
            dwell_left_banana: 0.0,
            dwell_right_deflecto: 0.0,
            portal_radius: DEFAULT_PORTAL_RADIUS,
        }
    }

    /// Resets all dwell timers to 0.0.
    pub fn reset(&mut self) {
        self.dwell_left_banana = 0.0;
        self.dwell_right_deflecto = 0.0;
    }

    /// Returns the currently selected game if dwell has reached 1.0.
    pub fn is_selected(&self) -> SelectedGame {
        if self.dwell_left_banana >= 0.999 {
            SelectedGame::BananaGrab
        } else if self.dwell_right_deflecto >= 0.999 {
            SelectedGame::DeflectoBot
        } else {
            SelectedGame::None
        }
    }

    /// Updates dwell timers based on input and returns current game selection status.
    pub fn update(&mut self, dt: f32, input: &PlayerInput) -> SelectedGame {
        let (left_active, right_active) = self.check_portal_contacts(input);

        // Update left portal dwell timer
        if left_active {
            let increment = if DWELL_TARGET_SECONDS > 0.0 {
                dt / DWELL_TARGET_SECONDS
            } else {
                1.0
            };
            let next = self.dwell_left_banana + increment;
            self.dwell_left_banana = if (next - 1.0).abs() < 1e-5 || next >= 1.0 {
                1.0
            } else {
                next
            };
        } else {
            // Smoothly decay when hand leaves portal
            self.dwell_left_banana = (self.dwell_left_banana - dt * 2.0).max(0.0);
        }

        // Update right portal dwell timer
        if right_active {
            let increment = if DWELL_TARGET_SECONDS > 0.0 {
                dt / DWELL_TARGET_SECONDS
            } else {
                1.0
            };
            let next = self.dwell_right_deflecto + increment;
            self.dwell_right_deflecto = if (next - 1.0).abs() < 1e-5 || next >= 1.0 {
                1.0
            } else {
                next
            };
        } else {
            // Smoothly decay when hand leaves portal
            self.dwell_right_deflecto = (self.dwell_right_deflecto - dt * 2.0).max(0.0);
        }

        self.is_selected()
    }

    /// Checks if confident wrists from any player are within the left or right portal radius.
    pub fn check_portal_contacts(&self, input: &PlayerInput) -> (bool, bool) {
        let mut left_active = false;
        let mut right_active = false;
        let r_sq = self.portal_radius * self.portal_radius;

        for player in &input.players {
            let wrists = [
                &player.keypoints[COCO_LEFT_WRIST],
                &player.keypoints[COCO_RIGHT_WRIST],
            ];

            for wrist in wrists {
                if !wrist.is_confident(MIN_KEYPOINT_CONFIDENCE) {
                    continue;
                }

                // Check distance to Left Portal (Banana Grab)
                let dlx = wrist.x - LEFT_PORTAL_CENTER.0;
                let dly = wrist.y - LEFT_PORTAL_CENTER.1;
                if dlx * dlx + dly * dly <= r_sq {
                    left_active = true;
                }

                // Check distance to Right Portal (Deflecto-Bot)
                let drx = wrist.x - RIGHT_PORTAL_CENTER.0;
                let dry = wrist.y - RIGHT_PORTAL_CENTER.1;
                if drx * drx + dry * dry <= r_sq {
                    right_active = true;
                }
            }
        }

        (left_active, right_active)
    }

    /// Renders the portal rings, labels, and dwell fill animation using CanvasPort.
    pub fn draw(&self, ctx: &mut RenderContext) {
        let sw = ctx.screen_width;
        let sh = ctx.screen_height;

        // 1. Draw Title and Subtitle HUD at top center
        let title = "MOTION ARCADE";
        let sub = "CHOOSE YOUR GAME - HOLD HAND IN PORTAL";
        ctx.draw
            .draw_text(title, sw * 0.5 - 180.0, 70.0, 48.0, [0.95, 0.98, 1.0, 0.95]);
        ctx.draw
            .draw_text(sub, sw * 0.5 - 250.0, 110.0, 24.0, [0.4, 0.8, 1.0, 0.75]);

        // 2. Draw Left Portal (Jungle Banana Grab)
        let (lx, ly) = (LEFT_PORTAL_CENTER.0 * sw, LEFT_PORTAL_CENTER.1 * sh);
        let portal_r = self.portal_radius * sh.min(sw);

        // Left base rings (Jungle Green & Warm Gold)
        ctx.draw
            .draw_ring(lx, ly, portal_r * 1.08, 2.0, [0.1, 0.8, 0.35, 0.3]);
        ctx.draw
            .draw_ring(lx, ly, portal_r, 4.0, [0.2, 0.9, 0.4, 0.75]);
        ctx.draw
            .draw_ring(lx, ly, portal_r * 0.92, 1.5, [1.0, 0.75, 0.1, 0.5]);
        ctx.draw
            .draw_circle(lx, ly, portal_r * 0.85, [0.05, 0.25, 0.1, 0.25]);

        // Banana Grab portal labels
        ctx.draw
            .draw_text("BANANA GRAB", lx - 95.0, ly - 20.0, 30.0, [1.0, 0.9, 0.2, 0.95]);
        ctx.draw
            .draw_text("JUNGLE JUMP", lx - 75.0, ly + 15.0, 22.0, [0.4, 0.95, 0.5, 0.85]);

        // Left dwell radial fill ring
        if self.dwell_left_banana > 0.0 {
            // Charging core
            ctx.draw.draw_circle(
                lx,
                ly,
                portal_r * self.dwell_left_banana * 0.85,
                [1.0, 0.8, 0.2, 0.35 * self.dwell_left_banana],
            );

            // Radial progress arc
            draw_radial_arc(
                ctx.draw,
                lx,
                ly,
                portal_r * 1.04,
                self.dwell_left_banana,
                6.0,
                [1.0, 0.85, 0.2, 0.95],
            );

            let pct_text = format!(
                "{:.0}%",
                (self.dwell_left_banana * 100.0).clamp(0.0, 100.0)
            );
            ctx.draw
                .draw_text(&pct_text, lx - 25.0, ly + 55.0, 26.0, [1.0, 1.0, 1.0, 0.95]);
        } else {
            ctx.draw
                .draw_text("REACH HERE", lx - 60.0, ly + 55.0, 20.0, [0.6, 0.85, 0.6, 0.6]);
        }

        // 3. Draw Right Portal (Deflecto-Bot Math)
        let (rx, ry) = (RIGHT_PORTAL_CENTER.0 * sw, RIGHT_PORTAL_CENTER.1 * sh);

        // Right base rings (Cyber Cyan & Laser Gold)
        ctx.draw
            .draw_ring(rx, ry, portal_r * 1.08, 2.0, [0.0, 0.75, 1.0, 0.3]);
        ctx.draw
            .draw_ring(rx, ry, portal_r, 4.0, [0.1, 0.88, 1.0, 0.8]);
        ctx.draw
            .draw_ring(rx, ry, portal_r * 0.92, 1.5, [1.0, 0.85, 0.2, 0.5]);
        ctx.draw
            .draw_circle(rx, ry, portal_r * 0.85, [0.02, 0.15, 0.3, 0.25]);

        // Deflecto-Bot portal labels
        ctx.draw
            .draw_text("DEFLECTO-BOT", rx - 105.0, ry - 20.0, 30.0, [0.2, 0.95, 1.0, 0.95]);
        ctx.draw
            .draw_text("CYBER MATH", rx - 75.0, ry + 15.0, 22.0, [1.0, 0.88, 0.3, 0.85]);

        // Right dwell radial fill ring
        if self.dwell_right_deflecto > 0.0 {
            // Charging core
            ctx.draw.draw_circle(
                rx,
                ry,
                portal_r * self.dwell_right_deflecto * 0.85,
                [0.1, 0.85, 1.0, 0.35 * self.dwell_right_deflecto],
            );

            // Radial progress arc
            draw_radial_arc(
                ctx.draw,
                rx,
                ry,
                portal_r * 1.04,
                self.dwell_right_deflecto,
                6.0,
                [0.2, 0.95, 1.0, 0.95],
            );

            let pct_text = format!(
                "{:.0}%",
                (self.dwell_right_deflecto * 100.0).clamp(0.0, 100.0)
            );
            ctx.draw
                .draw_text(&pct_text, rx - 25.0, ry + 55.0, 26.0, [1.0, 1.0, 1.0, 0.95]);
        } else {
            ctx.draw
                .draw_text("REACH HERE", rx - 60.0, ry + 55.0, 20.0, [0.6, 0.8, 1.0, 0.6]);
        }

        // 4. Instructions footer
        let exit_hint = "EXIT ANY GAME: Hold T-Pose or Raise Both Hands for 3 seconds";
        ctx.draw.draw_text(
            exit_hint,
            sw * 0.5 - 280.0,
            sh - 45.0,
            20.0,
            [0.65, 0.75, 0.85, 0.7],
        );
    }
}

/// Helper to draw a clockwise radial progress arc starting at 12 o'clock (-PI/2).
fn draw_radial_arc(
    draw: &mut dyn CanvasPort,
    cx: f32,
    cy: f32,
    radius: f32,
    progress: f32,
    thickness: f32,
    color: [f32; 4],
) {
    let p = progress.clamp(0.0, 1.0);
    if p <= 0.001 {
        return;
    }

    let segments = ((p * 48.0).ceil() as usize).max(2);
    let start_angle = -FRAC_PI_2;
    let total_angle = TAU * p;
    let step = total_angle / (segments as f32);

    for i in 0..segments {
        let a1 = start_angle + (i as f32) * step;
        let a2 = start_angle + ((i + 1) as f32) * step;
        let x1 = cx + a1.cos() * radius;
        let y1 = cy + a1.sin() * radius;
        let x2 = cx + a2.cos() * radius;
        let y2 = cy + a2.sin() * radius;

        // Outer glow
        draw.draw_line(
            x1,
            y1,
            x2,
            y2,
            thickness * 1.5,
            [color[0], color[1], color[2], color[3] * 0.4],
        );
        // Sharp inner laser
        draw.draw_line(x1, y1, x2, y2, thickness, color);
    }

    // Spark node at leading tip
    let tip_angle = start_angle + total_angle;
    let tip_x = cx + tip_angle.cos() * radius;
    let tip_y = cy + tip_angle.sin() * radius;
    draw.draw_circle(tip_x, tip_y, thickness * 1.2, [1.0, 1.0, 1.0, 0.95]);
}

/// Checks if both wrists are raised above upper body (shoulders or nose).
pub fn both_arms_up(skeleton: &Skeleton) -> bool {
    let lw = &skeleton.keypoints[COCO_LEFT_WRIST];
    let rw = &skeleton.keypoints[COCO_RIGHT_WRIST];

    if !lw.is_confident(MIN_KEYPOINT_CONFIDENCE) || !rw.is_confident(MIN_KEYPOINT_CONFIDENCE) {
        return false;
    }

    let nose = &skeleton.keypoints[COCO_NOSE];
    if nose.is_confident(0.15) && lw.y < nose.y && rw.y < nose.y {
        return true;
    }

    let ls = &skeleton.keypoints[COCO_LEFT_SHOULDER];
    let rs = &skeleton.keypoints[COCO_RIGHT_SHOULDER];
    if ls.is_confident(0.15) && rs.is_confident(0.15) {
        if lw.y < ls.y && rw.y < rs.y {
            return true;
        }
    } else if (ls.is_confident(0.15) && lw.y < ls.y && rw.y < ls.y)
        || (rs.is_confident(0.15) && lw.y < rs.y && rw.y < rs.y)
    {
        return true;
    }

    // Fallback: check if both wrists are in upper 35% of the screen
    if !nose.is_confident(0.15) && !ls.is_confident(0.15) && !rs.is_confident(0.15) {
        return lw.y < 0.35 && rw.y < 0.35;
    }

    false
}

/// Checks if skeleton is holding a T-pose (arms spread horizontally at shoulder height).
pub fn is_t_pose(skeleton: &Skeleton) -> bool {
    let lw = &skeleton.keypoints[COCO_LEFT_WRIST];
    let rw = &skeleton.keypoints[COCO_RIGHT_WRIST];

    if !lw.is_confident(MIN_KEYPOINT_CONFIDENCE) || !rw.is_confident(MIN_KEYPOINT_CONFIDENCE) {
        return false;
    }

    // Horizontal arm span must be wide across torso
    let span = (lw.x - rw.x).abs();
    if span < 0.40 {
        return false;
    }

    // Determine shoulder height reference
    let ls = &skeleton.keypoints[COCO_LEFT_SHOULDER];
    let rs = &skeleton.keypoints[COCO_RIGHT_SHOULDER];
    let shoulder_y = if ls.is_confident(0.15) && rs.is_confident(0.15) {
        (ls.y + rs.y) * 0.5
    } else if ls.is_confident(0.15) {
        ls.y
    } else if rs.is_confident(0.15) {
        rs.y
    } else {
        0.35
    };

    // Wrists must be approximately aligned vertically with shoulder height (within +/- 0.14)
    let lw_diff = (lw.y - shoulder_y).abs();
    let rw_diff = (rw.y - shoulder_y).abs();

    lw_diff < 0.14 && rw_diff < 0.14
}

/// Checks if skeleton is executing an exit gesture (either both arms up or T-pose).
pub fn is_exit_gesture(skeleton: &Skeleton) -> bool {
    both_arms_up(skeleton) || is_t_pose(skeleton)
}

/// Checks if any player in the input is currently holding an exit gesture.
pub fn check_exit_gesture(input: &PlayerInput) -> bool {
    input.players.iter().any(is_exit_gesture)
}
