use core::game::{Game, GameStatus, RenderContext};
use core::pose::{PlayerInput, COCO_LEFT_WRIST, COCO_RIGHT_WRIST};
use learning::bridge::{LearningSessionPort, QuestionWave};
use std::f32::consts::TAU;

/// Number of particles spawned per orb explosion.
pub const PARTICLES_PER_EXPLOSION: usize = 30;

/// Minimum confidence required for a wrist keypoint to trigger deflection.
pub const MIN_WRIST_CONFIDENCE: f32 = 0.3;

/// Default flight duration in seconds for an orb to travel from horizon (0.0) to strike line (1.0).
pub const DEFAULT_ORB_SPEED: f32 = 0.5;

/// The four interaction zones on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quadrant {
    TopLeft,     // x: [0.0, 0.5], y: [0.0, 0.5]
    TopRight,    // x: [0.5, 1.0], y: [0.0, 0.5]
    BottomLeft,  // x: [0.0, 0.5], y: [0.5, 1.0]
    BottomRight, // x: [0.5, 1.0], y: [0.5, 1.0]
}

impl Quadrant {
    /// Maps normalized 2D point (x, y) to its corresponding quadrant.
    pub fn from_point(x: f32, y: f32) -> Option<Self> {
        let cx = x.clamp(0.0, 1.0);
        let cy = y.clamp(0.0, 1.0);
        if cy < 0.5 {
            if cx < 0.5 {
                Some(Quadrant::TopLeft)
            } else {
                Some(Quadrant::TopRight)
            }
        } else if cx < 0.5 {
            Some(Quadrant::BottomLeft)
        } else {
            Some(Quadrant::BottomRight)
        }
    }

    /// Center point of the quadrant in normalized [0.0, 1.0] coordinates.
    pub fn center(&self) -> (f32, f32) {
        match self {
            Quadrant::TopLeft => (0.25, 0.25),
            Quadrant::TopRight => (0.75, 0.25),
            Quadrant::BottomLeft => (0.25, 0.75),
            Quadrant::BottomRight => (0.75, 0.75),
        }
    }

    /// All four quadrants in clockwise order.
    pub fn all() -> [Quadrant; 4] {
        [
            Quadrant::TopLeft,
            Quadrant::TopRight,
            Quadrant::BottomRight,
            Quadrant::BottomLeft,
        ]
    }

    /// Converts a 0..=3 index to its corresponding quadrant.
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Quadrant::TopLeft),
            1 => Some(Quadrant::TopRight),
            2 => Some(Quadrant::BottomLeft),
            3 => Some(Quadrant::BottomRight),
            _ => None,
        }
    }

    /// Converts quadrant to 0..=3 index.
    pub fn to_u8(self) -> u8 {
        match self {
            Quadrant::TopLeft => 0,
            Quadrant::TopRight => 1,
            Quadrant::BottomLeft => 2,
            Quadrant::BottomRight => 3,
        }
    }
}

/// A high-speed plasma projectile flying toward the player.
#[derive(Debug, Clone, PartialEq)]
pub struct PlasmaOrb {
    pub id: u32,
    pub quadrant: Quadrant,
    pub value_text: String,
    pub is_correct: bool,
    pub progress: f32, // 0.0 (distant horizon) to 1.0 (strike line)
    pub radius: f32,
    pub item_id: String,
}

impl PlasmaOrb {
    pub fn new(
        id: u32,
        quadrant: Quadrant,
        value_text: impl Into<String>,
        is_correct: bool,
    ) -> Self {
        Self {
            id,
            quadrant,
            value_text: value_text.into(),
            is_correct,
            progress: 0.0,
            radius: 0.06,
            item_id: String::new(),
        }
    }

    pub fn with_item_id(mut self, item_id: impl Into<String>) -> Self {
        self.item_id = item_id.into();
        self
    }

    /// Current (x, y) coordinate in normalized space based on progress from horizon (0.5, 0.5) to quadrant target.
    pub fn position(&self) -> (f32, f32) {
        let (qx, qy) = self.quadrant.center();
        let hx = 0.5;
        let hy = 0.5;
        (
            hx + (qx - hx) * self.progress,
            hy + (qy - hy) * self.progress,
        )
    }
}

/// A visual impact particle spawned during orb deflection explosions.
#[derive(Debug, Clone, PartialEq)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub color: [f32; 4],
    pub radius: f32,
}

impl Particle {
    pub fn new(
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        max_lifetime: f32,
        color: [f32; 4],
        radius: f32,
    ) -> Self {
        Self {
            x,
            y,
            vx,
            vy,
            lifetime: 0.0,
            max_lifetime,
            color,
            radius,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.lifetime < self.max_lifetime
    }
}

/// State tracking for previous wrist quadrants to detect entry transitions.
#[derive(Debug, Clone, Default)]
struct PlayerWristState {
    player_id: u8,
    left_quadrant: Option<Quadrant>,
    right_quadrant: Option<Quadrant>,
}

/// Flagship Deflecto-Bot game implementation supporting both procedural arcade and educational modes.
pub struct DeflectoBot {
    pub score: u32,
    pub orbs: Vec<PlasmaOrb>,
    pub particles: Vec<Particle>,
    pub punch_intensity: f32,
    pub status: GameStatus,
    next_orb_id: u32,
    spawn_timer: f32,
    spawn_interval: f32,
    orb_speed: f32,
    wrist_states: Vec<PlayerWristState>,
    quadrant_cycle_idx: usize,
    total_time: f32,
    session: Option<Box<dyn LearningSessionPort>>,
    active_wave: Option<QuestionWave>,
}

impl std::fmt::Debug for DeflectoBot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeflectoBot")
            .field("score", &self.score)
            .field("orbs", &self.orbs)
            .field("particles", &self.particles)
            .field("punch_intensity", &self.punch_intensity)
            .field("status", &self.status)
            .field("spawn_interval", &self.spawn_interval)
            .field("orb_speed", &self.orb_speed)
            .field("active_wave", &self.active_wave)
            .field("is_learning_mode", &self.session.is_some())
            .finish()
    }
}

impl Default for DeflectoBot {
    fn default() -> Self {
        Self::new()
    }
}

impl DeflectoBot {
    /// Creates a standard arcade bot (numbers are procedural, no learning pack).
    pub fn new_arcade() -> Self {
        Self::new()
    }

    /// Creates an educational bot wired directly to a learning session.
    pub fn new_learning(session: Box<dyn LearningSessionPort>) -> Self {
        let mut bot = Self::new();
        bot.session = Some(session);
        bot.spawn_interval = 1.6;
        bot
    }

    /// Creates a default standalone bot.
    pub fn new() -> Self {
        Self {
            score: 0,
            orbs: Vec::new(),
            particles: Vec::new(),
            punch_intensity: 0.0,
            status: GameStatus::Running,
            next_orb_id: 1,
            spawn_timer: 0.0,
            spawn_interval: 1.6,
            orb_speed: DEFAULT_ORB_SPEED,
            wrist_states: Vec::new(),
            quadrant_cycle_idx: 0,
            total_time: 0.0,
            session: None,
            active_wave: None,
        }
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn orbs(&self) -> &[PlasmaOrb] {
        &self.orbs
    }

    pub fn orbs_mut(&mut self) -> &mut Vec<PlasmaOrb> {
        &mut self.orbs
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    pub fn punch_intensity(&self) -> f32 {
        self.punch_intensity
    }

    pub fn is_learning_mode(&self) -> bool {
        self.session.is_some()
    }

    pub fn session(&self) -> Option<&dyn LearningSessionPort> {
        self.session.as_deref()
    }

    pub fn session_mut(&mut self) -> Option<&mut dyn LearningSessionPort> {
        self.session.as_deref_mut()
    }

    pub fn active_wave(&self) -> Option<&QuestionWave> {
        self.active_wave.as_ref()
    }

    pub fn current_prompt(&self) -> Option<String> {
        if let Some(session) = &self.session {
            session.current_prompt()
        } else {
            self.active_wave.as_ref().map(|w| w.prompt.clone())
        }
    }

    pub fn set_spawn_interval(&mut self, interval: f32) {
        self.spawn_interval = interval;
    }

    pub fn set_orb_speed(&mut self, speed: f32) {
        self.orb_speed = speed;
    }

    pub fn screen_shake(&self) -> (f32, f32) {
        if self.punch_intensity <= 0.001 {
            (0.0, 0.0)
        } else {
            let shake = self.punch_intensity * 14.0;
            let sx = (self.total_time * 55.0).sin() * shake;
            let sy = (self.total_time * 48.0).cos() * shake;
            (sx, sy)
        }
    }

    /// Spawns a new orb in the specified quadrant.
    pub fn spawn_orb(
        &mut self,
        quadrant: Quadrant,
        value_text: impl Into<String>,
        is_correct: bool,
    ) -> u32 {
        let id = self.next_orb_id;
        self.next_orb_id = self.next_orb_id.wrapping_add(1);
        self.orbs
            .push(PlasmaOrb::new(id, quadrant, value_text, is_correct));
        id
    }

    /// Spawns the next procedural orb cycling across the four quadrants.
    pub fn spawn_next_orb(&mut self) -> u32 {
        let quadrants = Quadrant::all();
        let quadrant = quadrants[self.quadrant_cycle_idx % quadrants.len()];
        self.quadrant_cycle_idx = self.quadrant_cycle_idx.wrapping_add(1);
        self.spawn_orb(quadrant, "", true)
    }

    /// Spawns a question wave from the learning session, populating 1 correct orb and distractors.
    pub fn spawn_question_wave(&mut self) -> bool {
        if let Some(session) = &mut self.session {
            if let Some(wave) = session.next_question_wave() {
                for choice in &wave.choices {
                    let quadrant = match Quadrant::from_u8(choice.target_quadrant) {
                        Some(q) => q,
                        None => Quadrant::TopLeft,
                    };
                    let id = self.next_orb_id;
                    self.next_orb_id = self.next_orb_id.wrapping_add(1);
                    let mut orb =
                        PlasmaOrb::new(id, quadrant, choice.text.clone(), choice.is_correct);
                    orb.item_id = wave.item_id.clone();
                    self.orbs.push(orb);
                }
                self.active_wave = Some(wave);
                return true;
            }
        }
        false
    }

    /// Triggers an orb explosion into 30 particles with screen punch effect.
    pub fn explode_orb(&mut self, orb: &PlasmaOrb) {
        let (ox, oy) = orb.position();
        self.score = self.score.saturating_add(1);
        self.punch_intensity = 1.0;

        for i in 0..PARTICLES_PER_EXPLOSION {
            let angle = (i as f32) * (TAU / (PARTICLES_PER_EXPLOSION as f32));
            let speed_variation = 0.30 + ((i % 5) as f32) * 0.07;
            let vx = angle.cos() * speed_variation;
            let vy = angle.sin() * speed_variation;
            let max_lifetime = 0.45 + ((i % 3) as f32) * 0.12;

            // Golden starburst / bright plasma yellow
            let color = [1.0, 0.88, 0.25, 1.0];

            self.particles
                .push(Particle::new(ox, oy, vx, vy, max_lifetime, color, 0.012));
        }
    }

    /// Triggers a gentle soft red dissolve effect when an incorrect orb is deflected.
    pub fn dissolve_orb(&mut self, orb: &PlasmaOrb) {
        let (ox, oy) = orb.position();
        self.punch_intensity = (self.punch_intensity + 0.12).min(0.30);

        const DISSOLVE_PARTICLES: usize = 20;
        for i in 0..DISSOLVE_PARTICLES {
            let angle = (i as f32) * (TAU / (DISSOLVE_PARTICLES as f32));
            let speed = 0.09 + ((i % 4) as f32) * 0.03;
            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed - 0.04; // subtle upward drift
            let max_lifetime = 0.55 + ((i % 3) as f32) * 0.12;

            // Soft crimson red dissolve
            let color = [0.95, 0.28, 0.28, 0.75];

            self.particles
                .push(Particle::new(ox, oy, vx, vy, max_lifetime, color, 0.010));
        }
    }

    /// Checks if a wrist entered a quadrant or if a confident wrist is present in a hittable quadrant.
    fn check_quadrant_hits(&mut self, input: &PlayerInput) -> Vec<usize> {
        let mut hit_indices = Vec::new();

        // Collect current confident wrist quadrants for all players
        let mut entered_quadrants = Vec::new();
        let mut active_quadrants = Vec::new();

        for player in &input.players {
            let left_wrist = &player.keypoints[COCO_LEFT_WRIST];
            let right_wrist = &player.keypoints[COCO_RIGHT_WRIST];

            let curr_left = if left_wrist.is_confident(MIN_WRIST_CONFIDENCE) {
                Quadrant::from_point(left_wrist.x, left_wrist.y)
            } else {
                None
            };

            let curr_right = if right_wrist.is_confident(MIN_WRIST_CONFIDENCE) {
                Quadrant::from_point(right_wrist.x, right_wrist.y)
            } else {
                None
            };

            // Find or insert previous wrist state for this player
            let prev_state = self
                .wrist_states
                .iter()
                .find(|s| s.player_id == player.id)
                .cloned();

            let (prev_left, prev_right) = match prev_state {
                Some(state) => (state.left_quadrant, state.right_quadrant),
                None => (None, None),
            };

            // Detect if wrist transitioned/entered into a quadrant
            if let Some(ql) = curr_left {
                active_quadrants.push(ql);
                if prev_left != Some(ql) {
                    entered_quadrants.push(ql);
                }
            }

            if let Some(qr) = curr_right {
                active_quadrants.push(qr);
                if prev_right != Some(qr) {
                    entered_quadrants.push(qr);
                }
            }
        }

        // Update stored wrist states for next frame
        self.wrist_states.clear();
        for player in &input.players {
            let left_wrist = &player.keypoints[COCO_LEFT_WRIST];
            let right_wrist = &player.keypoints[COCO_RIGHT_WRIST];

            let curr_left = if left_wrist.is_confident(MIN_WRIST_CONFIDENCE) {
                Quadrant::from_point(left_wrist.x, left_wrist.y)
            } else {
                None
            };

            let curr_right = if right_wrist.is_confident(MIN_WRIST_CONFIDENCE) {
                Quadrant::from_point(right_wrist.x, right_wrist.y)
            } else {
                None
            };

            self.wrist_states.push(PlayerWristState {
                player_id: player.id,
                left_quadrant: curr_left,
                right_quadrant: curr_right,
            });
        }

        // Determine which orbs were struck
        for (idx, orb) in self.orbs.iter().enumerate() {
            let entered = entered_quadrants.contains(&orb.quadrant);
            let active = active_quadrants.contains(&orb.quadrant);

            // Deflect if wrist just punched/entered the quadrant,
            // or if wrist is holding guard in the quadrant as orb reaches strike zone (progress >= 0.4)
            if entered || (active && orb.progress >= 0.4) {
                hit_indices.push(idx);
            }
        }

        hit_indices
    }
}

impl Game for DeflectoBot {
    fn id(&self) -> &'static str {
        "deflecto_bot"
    }

    fn name(&self) -> &'static str {
        "Deflecto-Bot"
    }

    fn init(&mut self) {
        self.score = 0;
        self.orbs.clear();
        self.particles.clear();
        self.punch_intensity = 0.0;
        self.status = GameStatus::Running;
        self.next_orb_id = 1;
        self.spawn_timer = 0.0;
        self.wrist_states.clear();
        self.quadrant_cycle_idx = 0;
        self.total_time = 0.0;
        self.active_wave = None;

        // Spawn first orb or question wave immediately
        if self.session.is_some() {
            self.spawn_question_wave();
        } else {
            self.spawn_next_orb();
        }
    }

    fn update(&mut self, dt: f32, input: &PlayerInput) -> GameStatus {
        self.total_time += dt;

        // 1. Advance existing orbs
        for orb in &mut self.orbs {
            orb.progress += self.orb_speed * dt;
        }

        // 2. Check wrist collisions in quadrants
        let hit_indices = self.check_quadrant_hits(input);

        let mut answered_correctly = false;
        let mut correct_wave_id = String::new();

        // Process hit orbs in reverse order so indices remain valid
        for &idx in hit_indices.iter().rev() {
            if idx < self.orbs.len() {
                let orb = self.orbs[idx].clone();

                let is_correct = if let Some(session) = &mut self.session {
                    let item_id = if !orb.item_id.is_empty() {
                        &orb.item_id
                    } else if let Some(wave) = &self.active_wave {
                        &wave.item_id
                    } else {
                        ""
                    };
                    session.evaluate_choice(item_id, &orb.value_text)
                } else {
                    orb.is_correct
                };

                if is_correct {
                    let removed = self.orbs.remove(idx);
                    self.explode_orb(&removed);
                    answered_correctly = true;
                    correct_wave_id = removed.item_id;
                } else {
                    let removed = self.orbs.remove(idx);
                    self.dissolve_orb(&removed);
                }
            }
        }

        // If in learning mode and correct orb was hit:
        // Clear remaining distractor orbs for this question wave and reset spawn timer
        if answered_correctly && self.session.is_some() {
            if !correct_wave_id.is_empty() {
                self.orbs.retain(|o| o.item_id != correct_wave_id);
            } else {
                self.orbs.clear();
            }
            self.spawn_timer = 0.0;
        }

        // 3. Remove missed orbs that passed beyond the strike line
        self.orbs.retain(|orb| orb.progress <= 1.25);

        // 4. Update particles
        for particle in &mut self.particles {
            particle.x += particle.vx * dt;
            particle.y += particle.vy * dt;
            particle.lifetime += dt;
        }
        self.particles.retain(|p| p.is_alive());

        // 5. Decay screen punch intensity
        self.punch_intensity = (self.punch_intensity - dt * 5.0).max(0.0);

        // 6. Spawn procedural incoming orbs or question waves
        self.spawn_timer += dt;
        if self.session.is_some() {
            if self.orbs.is_empty() && self.spawn_timer >= self.spawn_interval {
                self.spawn_timer = 0.0;
                self.spawn_question_wave();
            }
        } else if self.spawn_timer >= self.spawn_interval {
            self.spawn_timer = 0.0;
            self.spawn_next_orb();
        }

        self.status
    }

    fn draw(&self, ctx: &mut RenderContext) {
        let sw = ctx.screen_width;
        let sh = ctx.screen_height;
        let (shake_x, shake_y) = self.screen_shake();

        // 1. Draw quadrant boundary guides
        let line_color = [0.15, 0.45, 0.85, 0.18];
        ctx.draw.draw_line(
            sw * 0.5 + shake_x,
            shake_y,
            sw * 0.5 + shake_x,
            sh + shake_y,
            1.5,
            line_color,
        );
        ctx.draw.draw_line(
            shake_x,
            sh * 0.5 + shake_y,
            sw + shake_x,
            sh * 0.5 + shake_y,
            1.5,
            line_color,
        );

        // Draw quadrant target indicators
        for q in &Quadrant::all() {
            let (cx, cy) = q.center();
            let qx = cx * sw + shake_x;
            let qy = cy * sh + shake_y;
            ctx.draw.draw_ring(qx, qy, 48.0, 1.5, [0.1, 0.6, 0.9, 0.25]);
            ctx.draw.draw_ring(qx, qy, 52.0, 1.0, [0.1, 0.6, 0.9, 0.12]);
        }

        // 2. Draw flying orbs
        for orb in &self.orbs {
            let (ox, oy) = orb.position();
            let sx = ox * sw + shake_x;
            let sy = oy * sh + shake_y;

            // Perspective scaling: grows larger as it approaches strike line
            let scale = 0.35 + 0.65 * orb.progress.clamp(0.0, 1.2);
            let base_radius = orb.radius * sh.min(sw);
            let r = base_radius * scale;

            // Outer glow ring
            ctx.draw
                .draw_ring(sx, sy, r * 1.35, 4.0, [0.1, 0.8, 1.0, 0.45]);
            // Main plasma body
            ctx.draw.draw_circle(sx, sy, r, [0.2, 0.85, 1.0, 0.85]);
            // Inner core
            ctx.draw
                .draw_circle(sx, sy, r * 0.45, [1.0, 1.0, 1.0, 0.95]);

            // Optional text label
            if !orb.value_text.is_empty() {
                let font_size = (26.0 * scale).clamp(16.0, 36.0);
                let text_offset_x = (orb.value_text.len() as f32 * font_size * 0.28).max(10.0);
                ctx.draw.draw_text(
                    &orb.value_text,
                    sx - text_offset_x,
                    sy + font_size * 0.35,
                    font_size,
                    [1.0, 1.0, 1.0, 1.0],
                );
            }
        }

        // 3. Draw explosion particles
        for p in &self.particles {
            let px = p.x * sw + shake_x;
            let py = p.y * sh + shake_y;
            let alpha = (1.0 - p.lifetime / p.max_lifetime).clamp(0.0, 1.0);
            let mut color = p.color;
            color[3] *= alpha;

            let pr = (p.radius * sw).max(3.0) * (0.4 + 0.6 * alpha);
            ctx.draw.draw_circle(px, py, pr, color);
        }

        // 4. Draw screen punch impact ring
        if self.punch_intensity > 0.01 {
            let shockwave_r = (1.0 - self.punch_intensity) * (sw * 0.45);
            let alpha = self.punch_intensity * 0.6;
            ctx.draw.draw_ring(
                sw * 0.5 + shake_x,
                sh * 0.5 + shake_y,
                shockwave_r,
                4.0,
                [1.0, 0.9, 0.3, alpha],
            );
        }

        // 5. Draw HUD header, score, and active prompt
        let score_text = format!("SCORE: {:04}", self.score);
        ctx.draw
            .draw_text(&score_text, 36.0, 52.0, 34.0, [1.0, 0.9, 0.25, 1.0]);

        if let Some(prompt) = self.current_prompt() {
            // Display active question prompt at top center
            let font_size = 38.0;
            let est_half_width = prompt.len() as f32 * font_size * 0.28;
            let px = (sw * 0.5 - est_half_width).max(180.0);
            ctx.draw
                .draw_text(&prompt, px, 52.0, font_size, [1.0, 1.0, 1.0, 1.0]);

            // Display session statistics on top right if in learning mode
            if let Some(session) = &self.session {
                let (correct, total) = session.session_stats();
                let stats_text = format!("{}/{} CORRECT", correct, total);
                ctx.draw
                    .draw_text(&stats_text, sw - 230.0, 52.0, 24.0, [0.2, 0.85, 1.0, 0.9]);
            }
        } else {
            // Standard Arcade Mode title
            ctx.draw.draw_text(
                "DEFLECTO-BOT",
                sw * 0.5 - 95.0,
                48.0,
                28.0,
                [0.2, 0.85, 1.0, 0.8],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::pose::{Keypoint, Skeleton, KEYPOINT_COUNT};
    use learning::bridge::{LearningSessionPort, OrbChoice, QuestionWave};

    fn make_input_with_wrist(quadrant: Quadrant) -> PlayerInput {
        let (wx, wy) = quadrant.center();
        let mut keypoints = [Keypoint::ZERO; KEYPOINT_COUNT];
        // Right wrist inside the specified quadrant with high confidence
        keypoints[COCO_RIGHT_WRIST] = Keypoint::new(wx, wy, 0.9);
        let skel = Skeleton::new(1, keypoints);
        PlayerInput::new(vec![skel], 1.0)
    }

    struct MockSession {
        prompts: Vec<String>,
        calls: Vec<String>,
        evaluated: Vec<(String, String)>,
    }

    impl MockSession {
        fn new() -> Self {
            Self {
                prompts: vec!["2 + 3 = ?".to_string()],
                calls: Vec::new(),
                evaluated: Vec::new(),
            }
        }
    }

    impl LearningSessionPort for MockSession {
        fn current_prompt(&self) -> Option<String> {
            self.prompts.first().cloned()
        }

        fn next_question_wave(&mut self) -> Option<QuestionWave> {
            self.calls.push("next_wave".to_string());
            Some(QuestionWave {
                item_id: "q1".to_string(),
                prompt: "2 + 3 = ?".to_string(),
                correct_answer: "5".to_string(),
                choices: vec![
                    OrbChoice {
                        text: "5".to_string(),
                        is_correct: true,
                        target_quadrant: 0,
                    },
                    OrbChoice {
                        text: "4".to_string(),
                        is_correct: false,
                        target_quadrant: 1,
                    },
                ],
            })
        }

        fn evaluate_choice(&mut self, item_id: &str, chosen_text: &str) -> bool {
            self.evaluated
                .push((item_id.to_string(), chosen_text.to_string()));
            chosen_text == "5"
        }

        fn session_stats(&self) -> (u32, u32) {
            let correct = self.evaluated.iter().filter(|(_, ans)| ans == "5").count() as u32;
            (correct, self.evaluated.len() as u32)
        }
    }

    #[test]
    fn test_quadrant_from_point() {
        assert_eq!(Quadrant::from_point(0.2, 0.2), Some(Quadrant::TopLeft));
        assert_eq!(Quadrant::from_point(0.8, 0.2), Some(Quadrant::TopRight));
        assert_eq!(Quadrant::from_point(0.2, 0.8), Some(Quadrant::BottomLeft));
        assert_eq!(Quadrant::from_point(0.8, 0.8), Some(Quadrant::BottomRight));

        // Boundary cases clamped safely
        assert_eq!(Quadrant::from_point(0.0, 0.0), Some(Quadrant::TopLeft));
        assert_eq!(Quadrant::from_point(1.0, 1.0), Some(Quadrant::BottomRight));
    }

    #[test]
    fn test_orb_position_progression() {
        let mut orb = PlasmaOrb::new(1, Quadrant::TopLeft, "", true);
        assert_eq!(orb.progress, 0.0);

        // Horizon at (0.5, 0.5)
        let (hx, hy) = orb.position();
        assert!((hx - 0.5).abs() < 1e-5);
        assert!((hy - 0.5).abs() < 1e-5);

        // Target center at progress 1.0
        orb.progress = 1.0;
        let (tx, ty) = orb.position();
        let (cx, cy) = Quadrant::TopLeft.center();
        assert!((tx - cx).abs() < 1e-5);
        assert!((ty - cy).abs() < 1e-5);
    }

    #[test]
    fn test_orb_deflection_spawns_30_particles_and_increments_score() {
        let mut bot = DeflectoBot::new();
        bot.orbs.clear();
        bot.spawn_orb(Quadrant::TopLeft, "test", true);
        assert_eq!(bot.orbs.len(), 1);
        assert_eq!(bot.score(), 0);
        assert_eq!(bot.particles().len(), 0);

        let input = make_input_with_wrist(Quadrant::TopLeft);
        let status = bot.update(0.016, &input);

        assert_eq!(status, GameStatus::Running);
        assert_eq!(bot.score(), 1);
        assert_eq!(bot.orbs.len(), 0);
        assert_eq!(bot.particles().len(), PARTICLES_PER_EXPLOSION);
        assert_eq!(bot.particles().len(), 30);
        assert!(bot.punch_intensity() > 0.9);
    }

    #[test]
    fn test_screen_punch_intensity_decay() {
        let mut bot = DeflectoBot::new();
        bot.punch_intensity = 1.0;

        let empty_input = PlayerInput::empty(0.0);
        bot.update(0.1, &empty_input);

        assert!(bot.punch_intensity() < 1.0);
        assert!(bot.punch_intensity() >= 0.0);
    }

    #[test]
    fn test_particle_lifetime_decay() {
        let mut bot = DeflectoBot::new();
        bot.particles.push(Particle::new(
            0.5,
            0.5,
            0.1,
            0.1,
            0.2, // 0.2s max lifetime
            [1.0, 1.0, 1.0, 1.0],
            0.01,
        ));
        assert_eq!(bot.particles().len(), 1);

        let empty_input = PlayerInput::empty(0.0);
        bot.update(0.3, &empty_input);
        assert_eq!(bot.particles().len(), 0);
    }

    #[test]
    fn test_low_confidence_wrist_ignored() {
        let mut bot = DeflectoBot::new();
        bot.orbs.clear();
        bot.spawn_orb(Quadrant::TopRight, "", true);

        let mut keypoints = [Keypoint::ZERO; KEYPOINT_COUNT];
        keypoints[COCO_RIGHT_WRIST] = Keypoint::new(0.75, 0.25, 0.1);
        let skel = Skeleton::new(1, keypoints);
        let input = PlayerInput::new(vec![skel], 1.0);

        bot.update(0.016, &input);

        assert_eq!(bot.score(), 0);
        assert_eq!(bot.orbs.len(), 1);
        assert_eq!(bot.particles().len(), 0);
    }

    #[test]
    fn test_deflecto_bot_game_trait_implementation() {
        let mut bot = DeflectoBot::new();
        assert_eq!(bot.id(), "deflecto_bot");
        assert_eq!(bot.name(), "Deflecto-Bot");

        bot.init();
        assert_eq!(bot.score(), 0);
        assert_eq!(bot.orbs().len(), 1);
    }

    #[test]
    fn test_learning_mode_initialization_and_evaluation() {
        let session = Box::new(MockSession::new());
        let mut bot = DeflectoBot::new_learning(session);
        bot.init();

        assert!(bot.is_learning_mode());
        assert_eq!(bot.current_prompt(), Some("2 + 3 = ?".to_string()));
        assert_eq!(bot.orbs().len(), 2);

        // Quadrant 1 has distractor "4" (incorrect)
        let wrong_input = make_input_with_wrist(Quadrant::TopRight);
        bot.update(0.016, &wrong_input);

        // Incorrect deflection: score does not increase, soft dissolve particles spawned
        assert_eq!(bot.score(), 0);
        assert!(!bot.particles().is_empty());
        // Correct orb is still alive!
        assert_eq!(bot.orbs().len(), 1);
        assert_eq!(bot.orbs()[0].value_text, "5");

        // Now hit Quadrant 0 with correct answer "5"
        let correct_input = make_input_with_wrist(Quadrant::TopLeft);
        bot.update(0.016, &correct_input);

        // Correct deflection: score incremented, gold particles spawned, orbs cleared
        assert_eq!(bot.score(), 1);
        assert_eq!(bot.orbs().len(), 0);
    }
}
