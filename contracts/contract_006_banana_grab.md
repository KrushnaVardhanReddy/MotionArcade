# Contract 006: Jungle Banana Grab Game

**Contract ID:** `CONTRACT-GAME-006`  
**Crate:** `crates/games/banana_grab`  
**Guarantees:** Zero `.unwrap()`, implements `core::game::Game`, zero dependency on Deflecto-Bot or App main.

---

## 1. Game State Contract

```rust
use core::game::{Game, GameStatus, RenderContext};
use core::pose::PlayerInput;

pub struct BananaGrab {
    score: u32,
    bananas_collected: u32,
    fruits: Vec<FallingFruit>,
    particles: Vec<LeafParticle>,
    swinging_vine_x: f32,
}

pub struct FallingFruit {
    pub id: u32,
    pub x: f32, // [0.0, 1.0]
    pub y: f32, // [0.0, 1.0]
    pub speed: f32,
    pub value_text: String,
    pub is_target: bool,
    pub radius: f32,
}

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
```
