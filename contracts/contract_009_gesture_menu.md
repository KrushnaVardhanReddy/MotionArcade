# Contract 009: Gesture Portal Menu

**Contract ID:** `CONTRACT-MENU-009`  
**Crate:** `crates/app`  
**Guarantees:** Zero `.unwrap()`, 60 FPS dwell fill animation, hands-free operation.

---

```rust
use core::pose::PlayerInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedGame {
    None,
    DeflectoBot,
    BananaGrab,
}

pub struct GestureMenu {
    pub dwell_left_banana: f32, // 0.0 to 1.0 (1.0 = launch)
    pub dwell_right_deflecto: f32,
    pub portal_radius: f32,
}

impl GestureMenu {
    pub fn new() -> Self;
    pub fn update(&mut self, dt: f32, input: &PlayerInput) -> SelectedGame;
}
```

