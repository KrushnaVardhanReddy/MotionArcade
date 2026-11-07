//! Jungle Jump: Banana Grab
//!
//! An animal-themed motion arcade mini-game designed for young children (ages 4–7).
//! Features falling tropical fruits, high-reach fruit collection, cartoon leaf FX,
//! and vine ducking mechanics.

use audio::{AudioMixer, SoundEffect};
use core::game::{Game, GameStatus, RenderContext};
use core::pose::{Keypoint, PlayerInput, Skeleton};
use std::f32::consts::TAU;
use std::sync::Arc;

/// Minimum keypoint confidence for wrist tracking to register grabs.
pub const MIN_WRIST_CONFIDENCE: f32 = 0.25;

/// Reach radius for grabbing fruit in normalized coordinates.
pub const GRAB_RADIUS: f32 = 0.12;

/// Half-width of vine clearance zone in normalized coordinates.
pub const VINE_CLEARANCE_ZONE: f32 = 0.15;

/// Default fall speed for falling fruits (fraction of screen per second).
pub const DEFAULT_FRUIT_SPEED: f32 = 0.35;

/// Default sweep speed for the swinging vine.
pub const DEFAULT_VINE_SPEED: f32 = 0.40;

/// Number of leaf particles spawned when fruit is collected.
pub const PARTICLES_PER_COLLECTION: usize = 25;

/// Ratio threshold for squat/duck detection.
/// Standing tall is typically ~0.45 - 0.60; ducking drops below 0.42.
pub const DUCK_RATIO_THRESHOLD: f32 = 0.42;

/// A tropical fruit falling vertically from the canopy.
#[derive(Debug, Clone, PartialEq)]
pub struct FallingFruit {
    pub id: u32,
    pub x: f32, // [0.0, 1.0]
    pub y: f32, // [0.0, 1.0]
    pub speed: f32,
    pub value_text: String,
    pub is_target: bool,
    pub radius: f32,
}

impl FallingFruit {
    pub fn new(
        id: u32,
        x: f32,
        y: f32,
        speed: f32,
        value_text: impl Into<String>,
        is_target: bool,
    ) -> Self {
        Self {
            id,
            x,
            y,
            speed,
            value_text: value_text.into(),
            is_target,
            radius: 0.05,
        }
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn position(&self) -> (f32, f32) {
        (self.x, self.y)
    }
}

/// A tropical green leaf particle spawned when fruit is grabbed.
#[derive(Debug, Clone, PartialEq)]
pub struct LeafParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub rotation: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub color: [f32; 4],
}

impl LeafParticle {
    pub fn new(
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        rotation: f32,
        max_lifetime: f32,
        color: [f32; 4],
    ) -> Self {
        Self {
            x,
            y,
            vx,
            vy,
            rotation,
            lifetime: 0.0,
            max_lifetime,
            color,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.lifetime < self.max_lifetime
    }
}

/// Calculates the hip height ratio: hip height above ground divided by total standing span.
/// Drops significantly when player ducks or squats.
pub fn hip_height_ratio(skeleton: &Skeleton) -> f32 {
    let hip_y = if skeleton.left_hip().is_confident(0.1) && skeleton.right_hip().is_confident(0.1) {
        (skeleton.left_hip().y + skeleton.right_hip().y) * 0.5
    } else if skeleton.left_hip().is_confident(0.1) {
        skeleton.left_hip().y
    } else if skeleton.right_hip().is_confident(0.1) {
        skeleton.right_hip().y
    } else {
        0.6
    };

    let head_y = if skeleton.nose().is_confident(0.1) {
        skeleton.nose().y
    } else if skeleton.left_shoulder().is_confident(0.1)
        && skeleton.right_shoulder().is_confident(0.1)
    {
        (skeleton.left_shoulder().y + skeleton.right_shoulder().y) * 0.5
    } else {
        0.2
    };

    let ground_y =
        if skeleton.left_ankle().is_confident(0.1) && skeleton.right_ankle().is_confident(0.1) {
            (skeleton.left_ankle().y + skeleton.right_ankle().y) * 0.5
        } else if skeleton.left_ankle().is_confident(0.1) {
            skeleton.left_ankle().y
        } else if skeleton.right_ankle().is_confident(0.1) {
            skeleton.right_ankle().y
        } else {
            1.0
        };

    let total_span = (ground_y - head_y).abs();
    if total_span <= 0.05 {
        return 0.5;
    }
    let hip_above_ground = (ground_y - hip_y).max(0.0);
    (hip_above_ground / total_span).clamp(0.0, 1.0)
}

/// Checks if player is currently in a ducking or squatting posture.
pub fn is_ducking_pose(skeleton: &Skeleton) -> bool {
    hip_height_ratio(skeleton) < DUCK_RATIO_THRESHOLD
}

/// Checks if wrist keypoint is raised above upper-body height (shoulder or nose).
pub fn is_wrist_raised(skeleton: &Skeleton, is_right: bool) -> bool {
    let wrist = if is_right {
        skeleton.right_wrist()
    } else {
        skeleton.left_wrist()
    };
    if !wrist.is_confident(MIN_WRIST_CONFIDENCE) {
        return false;
    }

    let nose = skeleton.nose();
    if nose.is_confident(0.15) && wrist.y < nose.y {
        return true;
    }

    let ls = skeleton.left_shoulder();
    let rs = skeleton.right_shoulder();
    if ls.is_confident(0.15) && rs.is_confident(0.15) {
        if wrist.y < ls.y.min(rs.y) {
            return true;
        }
    } else if (ls.is_confident(0.15) && wrist.y < ls.y) || (rs.is_confident(0.15) && wrist.y < rs.y)
    {
        return true;
    }

    // Fallback if head/shoulders are not detected: check upper screen region
    if !nose.is_confident(0.15) && !ls.is_confident(0.15) && !rs.is_confident(0.15) {
        return wrist.y < 0.5;
    }

    false
}

/// Checks if a wrist position is close to a falling fruit in column or radius.
pub fn is_near_fruit(wrist: &Keypoint, fruit: &FallingFruit) -> bool {
    let dx = (wrist.x - fruit.x).abs();
    let dy = (wrist.y - fruit.y).abs();
    let reach = fruit.radius + GRAB_RADIUS;
    (dx * dx + dy * dy) <= (reach * reach) || (dx <= 0.14 && dy <= 0.22)
}

/// Jungle Jump: Banana Grab game state.
#[derive(Debug, Clone)]
pub struct BananaGrab {
    pub score: u32,
    pub bananas_collected: u32,
    pub fruits: Vec<FallingFruit>,
    pub particles: Vec<LeafParticle>,
    pub swinging_vine_x: f32,
    pub vine_speed: f32,
    pub vine_dir: f32,
    pub is_ducking: bool,
    pub vine_dodged: bool,
    pub vine_contact: bool,
    pub vine_hits: u32,
    pub vine_dodges: u32,
    pub status: GameStatus,
    pub next_fruit_id: u32,
    pub spawn_timer: f32,
    pub spawn_interval: f32,
    pub total_time: f32,
    pub audio_mixer: Option<Arc<AudioMixer>>,
}

impl Default for BananaGrab {
    fn default() -> Self {
        Self::new()
    }
}

impl BananaGrab {
    pub fn new() -> Self {
        let mut game = Self {
            score: 0,
            bananas_collected: 0,
            fruits: Vec::new(),
            particles: Vec::new(),
            swinging_vine_x: 0.5,
            vine_speed: DEFAULT_VINE_SPEED,
            vine_dir: 1.0,
            is_ducking: false,
            vine_dodged: false,
            vine_contact: false,
            vine_hits: 0,
            vine_dodges: 0,
            status: GameStatus::Running,
            next_fruit_id: 1,
            spawn_timer: 0.0,
            spawn_interval: 1.4,
            total_time: 0.0,
            audio_mixer: None,
        };
        game.init();
        game
    }

    /// Attaches an audio mixer for sound effects.
    pub fn with_audio(mut self, mixer: Arc<AudioMixer>) -> Self {
        self.audio_mixer = Some(mixer);
        self
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn bananas_collected(&self) -> u32 {
        self.bananas_collected
    }

    pub fn fruits(&self) -> &[FallingFruit] {
        &self.fruits
    }

    pub fn fruits_mut(&mut self) -> &mut Vec<FallingFruit> {
        &mut self.fruits
    }

    pub fn particles(&self) -> &[LeafParticle] {
        &self.particles
    }

    pub fn swinging_vine_x(&self) -> f32 {
        self.swinging_vine_x
    }

    pub fn is_ducking(&self) -> bool {
        self.is_ducking
    }

    pub fn status(&self) -> GameStatus {
        self.status
    }

    /// Spawns a falling fruit.
    pub fn spawn_fruit(
        &mut self,
        x: f32,
        y: f32,
        speed: f32,
        value_text: impl Into<String>,
        is_target: bool,
    ) -> u32 {
        let id = self.next_fruit_id;
        self.next_fruit_id = self.next_fruit_id.wrapping_add(1);
        self.fruits
            .push(FallingFruit::new(id, x, y, speed, value_text, is_target));
        id
    }

    /// Spawns procedural fruit across alternating canopy columns.
    pub fn spawn_procedural_fruit(&mut self) -> u32 {
        let columns = [0.18, 0.34, 0.50, 0.66, 0.82];
        let col_idx = (self.next_fruit_id as usize) % columns.len();
        let x = columns[col_idx];
        let fruit_icons = ["🍌", "🥭", "🥥", "🍌", "🍌"];
        let icon = fruit_icons[(self.next_fruit_id as usize) % fruit_icons.len()];
        let speed = DEFAULT_FRUIT_SPEED + (((self.next_fruit_id % 3) as f32) * 0.05);
        self.spawn_fruit(x, 0.0, speed, icon, true)
    }

    /// Spawns exactly 25 green tropical leaf particles in a circular burst.
    pub fn spawn_leaf_burst(&mut self, x: f32, y: f32) {
        let colors = [
            [0.18, 0.78, 0.25, 1.0], // Jungle emerald green
            [0.10, 0.85, 0.35, 1.0], // Vibrant tropical green
            [0.28, 0.82, 0.18, 1.0], // Tropical lime green
            [0.12, 0.65, 0.22, 1.0], // Lush canopy green
            [0.22, 0.72, 0.30, 1.0], // Forest leaf green
        ];

        for i in 0..PARTICLES_PER_COLLECTION {
            let angle = (i as f32) * (TAU / (PARTICLES_PER_COLLECTION as f32));
            let speed = 0.20 + ((i % 5) as f32) * 0.06;
            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed - 0.05; // gentle upward burst
            let rotation = (i as f32) * 0.35;
            let max_lifetime = 0.50 + ((i % 4) as f32) * 0.10;
            let color = colors[i % colors.len()];

            self.particles.push(LeafParticle::new(
                x,
                y,
                vx,
                vy,
                rotation,
                max_lifetime,
                color,
            ));
        }
    }

    /// Collects a fruit by index, incrementing score & bananas_collected and spawning 25 leaf particles.
    pub fn collect_fruit(&mut self, fruit_idx: usize) -> Option<FallingFruit> {
        if fruit_idx >= self.fruits.len() {
            return None;
        }
        let fruit = self.fruits.remove(fruit_idx);
        self.bananas_collected = self.bananas_collected.saturating_add(1);
        self.score = self.score.saturating_add(1);
        self.spawn_leaf_burst(fruit.x, fruit.y);

        if let Some(mixer) = &self.audio_mixer {
            mixer.play(SoundEffect::BananaChomp);
        }

        Some(fruit)
    }

    /// Checks which fruits are touched by confident raised hands.
    pub fn check_fruit_grabs(&self, input: &PlayerInput) -> Vec<usize> {
        let mut hit_indices = Vec::new();

        for (fruit_idx, fruit) in self.fruits.iter().enumerate() {
            let mut grabbed = false;

            for player in &input.players {
                // Check right wrist
                let rw = player.right_wrist();
                if is_wrist_raised(player, true) && is_near_fruit(&rw, fruit) {
                    grabbed = true;
                    break;
                }

                // Check left wrist
                let lw = player.left_wrist();
                if is_wrist_raised(player, false) && is_near_fruit(&lw, fruit) {
                    grabbed = true;
                    break;
                }
            }

            if grabbed {
                hit_indices.push(fruit_idx);
            }
        }

        hit_indices
    }
}

impl Game for BananaGrab {
    fn id(&self) -> &'static str {
        "banana_grab"
    }

    fn name(&self) -> &'static str {
        "Jungle Jump: Banana Grab"
    }

    fn init(&mut self) {
        self.score = 0;
        self.bananas_collected = 0;
        self.fruits.clear();
        self.particles.clear();
        self.swinging_vine_x = 0.5;
        self.vine_speed = DEFAULT_VINE_SPEED;
        self.vine_dir = 1.0;
        self.is_ducking = false;
        self.vine_dodged = false;
        self.vine_contact = false;
        self.vine_hits = 0;
        self.vine_dodges = 0;
        self.status = GameStatus::Running;
        self.next_fruit_id = 1;
        self.spawn_timer = 0.0;
        self.total_time = 0.0;

        // Spawn initial falling banana
        self.spawn_fruit(0.5, 0.0, DEFAULT_FRUIT_SPEED, "🍌", true);
    }

    fn update(&mut self, dt: f32, input: &PlayerInput) -> GameStatus {
        self.total_time += dt;

        // 1. Advance falling fruit positions
        for fruit in &mut self.fruits {
            fruit.y += fruit.speed * dt;
        }

        // 2. Move swinging vine across top area
        self.swinging_vine_x += self.vine_speed * self.vine_dir * dt;
        if self.swinging_vine_x >= 0.95 {
            self.swinging_vine_x = 0.95;
            self.vine_dir = -1.0;
        } else if self.swinging_vine_x <= 0.05 {
            self.swinging_vine_x = 0.05;
            self.vine_dir = 1.0;
        }

        // 3. Check ducking and vine collision
        let mut ducking_active = false;
        let mut vine_near_player = false;

        for player in &input.players {
            let ducking = is_ducking_pose(player);
            if ducking {
                ducking_active = true;
            }

            let (px, _) = player.root_position();
            let dx = (self.swinging_vine_x - px).abs();
            if dx <= VINE_CLEARANCE_ZONE {
                vine_near_player = true;
                if ducking {
                    if !self.vine_dodged {
                        self.vine_dodged = true;
                        self.vine_dodges = self.vine_dodges.saturating_add(1);
                        self.score = self.score.saturating_add(2); // bonus for dodge
                    }
                } else {
                    self.vine_contact = true;
                    self.vine_hits = self.vine_hits.saturating_add(1);
                }
            }
        }

        self.is_ducking = ducking_active;
        if !vine_near_player {
            self.vine_dodged = false;
            self.vine_contact = false;
        }

        // 4. Check fruit grabs (wrist near falling fruit)
        let hit_indices = self.check_fruit_grabs(input);
        for &idx in hit_indices.iter().rev() {
            self.collect_fruit(idx);
        }

        // 5. Clean up missed fruits that passed screen bottom
        self.fruits.retain(|f| f.y <= 1.15);

        // 6. Update leaf particles
        for p in &mut self.particles {
            p.x += p.vx * dt;
            p.y += p.vy * dt + 0.15 * dt; // gentle gravity
            p.rotation += 2.0 * dt;
            p.lifetime += dt;
        }
        self.particles.retain(|p| p.is_alive());

        // 7. Spawn procedural fruits on timer
        self.spawn_timer += dt;
        if self.spawn_timer >= self.spawn_interval {
            self.spawn_timer = 0.0;
            self.spawn_procedural_fruit();
        }

        self.status
    }

    fn draw(&self, ctx: &mut RenderContext) {
        let sw = ctx.screen_width;
        let sh = ctx.screen_height;

        // 1. Draw canopy vine guide line across top
        ctx.draw
            .draw_line(0.0, sh * 0.12, sw, sh * 0.12, 3.0, [0.15, 0.45, 0.15, 0.35]);

        let num_nodes = 8;
        for i in 0..num_nodes {
            let nx = ((i as f32) / ((num_nodes - 1) as f32)) * sw;
            ctx.draw
                .draw_circle(nx, sh * 0.12, 10.0, [0.1, 0.5, 0.2, 0.25]);
        }

        // 2. Draw swinging vine
        let vx = self.swinging_vine_x * sw;
        let vy = sh * 0.30;
        let anchor_x = sw * 0.5;
        let anchor_y = 0.0;

        ctx.draw
            .draw_line(anchor_x, anchor_y, vx, vy, 4.0, [0.35, 0.25, 0.12, 0.85]);
        ctx.draw.draw_circle(vx, vy, 16.0, [0.18, 0.65, 0.15, 0.95]);
        ctx.draw
            .draw_ring(vx, vy, 22.0, 2.0, [0.25, 0.82, 0.2, 0.6]);

        if self.is_ducking {
            ctx.draw.draw_ring(vx, vy, 28.0, 2.0, [0.3, 1.0, 0.4, 0.5]);
        }

        // 3. Draw falling fruits
        for fruit in &self.fruits {
            let fx = fruit.x * sw;
            let fy = fruit.y * sh;
            let fr = fruit.radius * sw.min(sh);

            // Outer glow ring
            ctx.draw
                .draw_ring(fx, fy, fr * 1.3, 2.5, [1.0, 0.9, 0.2, 0.4]);
            // Main fruit body (bright banana yellow)
            ctx.draw.draw_circle(fx, fy, fr, [1.0, 0.85, 0.15, 0.95]);
            // Inner highlight
            ctx.draw.draw_circle(
                fx - fr * 0.2,
                fy - fr * 0.2,
                fr * 0.4,
                [1.0, 0.98, 0.6, 0.9],
            );
            // Label / icon
            if !fruit.value_text.is_empty() {
                ctx.draw.draw_text(
                    &fruit.value_text,
                    fx - 10.0,
                    fy + 7.0,
                    22.0,
                    [0.2, 0.15, 0.05, 1.0],
                );
            }
        }

        // 4. Draw leaf particles
        for p in &self.particles {
            let px = p.x * sw;
            let py = p.y * sh;
            let alpha = (1.0 - p.lifetime / p.max_lifetime).clamp(0.0, 1.0);
            let mut color = p.color;
            color[3] *= alpha;

            let pr = 6.0 * (0.5 + 0.5 * alpha);
            ctx.draw.draw_circle(px, py, pr, color);

            // Leaf vein line with rotation
            let dx = p.rotation.cos() * pr * 1.3;
            let dy = p.rotation.sin() * pr * 1.3;
            ctx.draw.draw_line(
                px - dx,
                py - dy,
                px + dx,
                py + dy,
                1.5,
                [0.1, 0.4, 0.1, color[3]],
            );
        }

        // 5. HUD Header
        let bananas_text = format!("BANANAS: {:03}", self.bananas_collected);
        ctx.draw
            .draw_text(&bananas_text, 36.0, 50.0, 32.0, [1.0, 0.88, 0.2, 1.0]);

        let score_text = format!("SCORE: {:04}", self.score);
        ctx.draw
            .draw_text(&score_text, 36.0, 85.0, 24.0, [0.4, 0.95, 0.4, 0.9]);

        ctx.draw.draw_text(
            "JUNGLE JUMP",
            sw * 0.5 - 90.0,
            42.0,
            26.0,
            [0.2, 0.85, 0.3, 0.9],
        );

        if self.is_ducking {
            ctx.draw.draw_text(
                "DUCK! VINE DODGED!",
                sw * 0.5 - 110.0,
                95.0,
                22.0,
                [0.3, 1.0, 0.4, 0.95],
            );
        }
    }
}
