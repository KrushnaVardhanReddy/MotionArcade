use crate::ports::input::PlayerInput;

/// Lifecycle and completion status of a running game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    /// Active gameplay in progress.
    Running,
    /// Game paused (e.g. tracking loss, menu overlay).
    Paused,
    /// Round or challenge completed with final score.
    Completed { final_score: u32 },
}

/// Rendering context provided to games on each draw tick.
pub struct RenderContext<'a> {
    /// Native window / viewport width in physical pixels.
    pub screen_width: f32,
    /// Native window / viewport height in physical pixels.
    pub screen_height: f32,
    /// Elapsed delta time since last rendered frame in seconds.
    pub dt: f32,
    /// Dynamic rendering backend implementing fundamental 2D primitives.
    pub draw: &'a mut dyn CanvasPort,
}

impl<'a> RenderContext<'a> {
    /// Creates a new render context.
    pub fn new(
        screen_width: f32,
        screen_height: f32,
        dt: f32,
        draw: &'a mut dyn CanvasPort,
    ) -> Self {
        Self {
            screen_width,
            screen_height,
            dt,
            draw,
        }
    }
}

/// Hexagonal outbound rendering port isolating games from underlying graphics libraries (Macroquad, WGPU, etc.).
pub trait CanvasPort {
    /// Draws a solid filled circle.
    /// `color` is in normalized RGBA `[r, g, b, a]` where each component is `0.0..=1.0`.
    fn draw_circle(&mut self, x: f32, y: f32, radius: f32, color: [f32; 4]);

    /// Draws an outlined ring/circle.
    fn draw_ring(&mut self, x: f32, y: f32, radius: f32, thickness: f32, color: [f32; 4]);

    /// Draws a text string at the given position with the specified font size.
    fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: [f32; 4]);

    /// Draws a straight line segment between two 2D points.
    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32, color: [f32; 4]);
}

/// The core domain interface implemented by all MotionArcade games.
///
/// Guaranteed properties:
/// - Deterministic updates independent of rendering frame rate.
/// - Thread-safe (`Send + 'static`).
/// - Zero graphics engine dependencies in game domain logic.
pub trait Game: Send + 'static {
    /// Unique identifier slug for this game (e.g., "deflecto_bot").
    fn id(&self) -> &'static str;

    /// User-facing display title (e.g., "Deflecto-Bot").
    fn name(&self) -> &'static str;

    /// Initializes or resets the game state for a new round.
    fn init(&mut self);

    /// Advances game simulation state by delta time `dt` seconds given latest player input.
    fn update(&mut self, dt: f32, input: &PlayerInput) -> GameStatus;

    /// Renders current visual state through the canvas port.
    fn draw(&self, ctx: &mut RenderContext);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCanvas {
        circle_count: usize,
        ring_count: usize,
        text_count: usize,
        line_count: usize,
    }

    impl MockCanvas {
        fn new() -> Self {
            Self {
                circle_count: 0,
                ring_count: 0,
                text_count: 0,
                line_count: 0,
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

    struct DummyGame {
        score: u32,
    }

    impl Game for DummyGame {
        fn id(&self) -> &'static str {
            "dummy"
        }

        fn name(&self) -> &'static str {
            "Dummy Game"
        }

        fn init(&mut self) {
            self.score = 0;
        }

        fn update(&mut self, _dt: f32, _input: &PlayerInput) -> GameStatus {
            self.score += 10;
            if self.score >= 50 {
                GameStatus::Completed {
                    final_score: self.score,
                }
            } else {
                GameStatus::Running
            }
        }

        fn draw(&self, ctx: &mut RenderContext) {
            ctx.draw
                .draw_circle(100.0, 100.0, 20.0, [1.0, 0.0, 0.0, 1.0]);
            ctx.draw
                .draw_ring(100.0, 100.0, 25.0, 2.0, [0.0, 1.0, 0.0, 1.0]);
            ctx.draw
                .draw_text("Hello", 50.0, 50.0, 24.0, [1.0, 1.0, 1.0, 1.0]);
            ctx.draw
                .draw_line(0.0, 0.0, 100.0, 100.0, 2.0, [0.0, 0.0, 1.0, 1.0]);
        }
    }

    #[test]
    fn test_game_trait_lifecycle() {
        let mut game = DummyGame { score: 100 };
        game.init();
        assert_eq!(game.score, 0);

        let input = PlayerInput::empty(0.0);
        let status = game.update(0.016, &input);
        assert_eq!(status, GameStatus::Running);
        assert_eq!(game.score, 10);

        let mut canvas = MockCanvas::new();
        let mut ctx = RenderContext::new(800.0, 600.0, 0.016, &mut canvas);
        game.draw(&mut ctx);

        assert_eq!(canvas.circle_count, 1);
        assert_eq!(canvas.ring_count, 1);
        assert_eq!(canvas.text_count, 1);
        assert_eq!(canvas.line_count, 1);
    }
}
