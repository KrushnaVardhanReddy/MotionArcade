# Contract 003: Macroquad Native Window, Keypoint Visualizer & Deflecto-Bot Game Loop

**Contract ID:** `CONTRACT-APP-003`  
**Crate:** `crates/app` & `crates/games/deflecto_bot`  
**Consumers:** Engine runner, CLI  
**Guarantees:** 60 FPS locked, zero webview overhead, non-blocking input stream consumption.

---

## 1. Game Trait Contract (`crates/core/src/game.rs`)

```rust
use crate::ports::input::PlayerInput;

pub enum GameStatus {
    Running,
    Paused,
    Completed { final_score: u32 },
}

pub struct RenderContext<'a> {
    pub screen_width: f32,
    pub screen_height: f32,
    pub dt: f32,
    pub draw: &'a mut dyn CanvasPort,
}

pub trait CanvasPort {
    fn draw_circle(&mut self, x: f32, y: f32, radius: f32, color: [f32; 4]);
    fn draw_ring(&mut self, x: f32, y: f32, radius: f32, thickness: f32, color: [f32; 4]);
    fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: [f32; 4]);
    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: [f32; 4]);
}

pub trait Game: Send + 'static {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn init(&mut self);
    fn update(&mut self, dt: f32, input: &PlayerInput) -> GameStatus;
    fn draw(&self, ctx: &mut RenderContext);
}
```

---

## 2. Deflecto-Bot Quadrant Hitzone Contract

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quadrant {
    TopLeft,     // x: [0.0, 0.5], y: [0.0, 0.5]
    TopRight,    // x: [0.5, 1.0], y: [0.0, 0.5]
    BottomLeft,  // x: [0.0, 0.5], y: [0.5, 1.0]
    BottomRight, // x: [0.5, 1.0], y: [0.5, 1.0]
}

pub struct PlasmaOrb {
    pub id: u32,
    pub quadrant: Quadrant,
    pub value_text: String,
    pub is_correct: bool,
    pub progress: f32, // 0.0 (distant horizon) to 1.0 (strike line)
    pub radius: f32,
}
```

---

## 3. Particle System Contract

```rust
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
```

