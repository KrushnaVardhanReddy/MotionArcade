# MotionArcade

**Your body is the controller.** A camera-based platform where kids play games, learn, and move — without controllers, wearables, or required internet.

A child stands in front of any camera, sees themselves on a TV or monitor, and plays by moving. Math problems fly at them as glowing orbs — they deflect the right answer with their hand. Letter shapes appear as wall cutouts — they twist their body to fit through. The learning is invisible. The movement is the point.

---

## Target Audience

1. **Kids at Home (Ages 4–12)** — Primary focus. Play mode, learn mode, or both.
2. **Classrooms (K–5)** — Short active brain breaks, turn-based rotation, teacher summaries.
3. **Adult Fitness & Coaching** — Later expansion using the same pose engine for rep counting and workouts.

---

## Market Positioning (The "Nex Playground" Contrast)

Where products like **Nex Playground** proved high demand for camera-based family gaming through a **$200+ proprietary hardware console and recurring subscription**, MotionArcade takes an open, accessible, and education-first approach:

| Dimension | Dedicated Consoles (e.g. Nex Playground) | **MotionArcade** |
|---|---|---|
| **Hardware Required** | Proprietary $200+ console box | **Zero extra cost.** Runs on existing laptops, Chromebooks, or Android TVs with standard webcams |
| **Core Purpose** | Pure casual entertainment & licensed IP (Fruit Ninja, Peppa Pig) | **Learning-first through active movement** (math, phonics, reading, science) |
| **Classroom Fit** | Impractical (schools cannot buy 30 proprietary consoles) | **Native fit.** Runs in a browser tab (WASM) on school Chromebooks and projectors |
| **Content Extensibility** | Closed garden (studio licensing only) | **Open & moddable.** Teachers and parents can add curricula via simple TOML lesson packs |
| **Connectivity** | Cloud-dependent subscription model | **Offline-first.** Runs 100% air-gapped without school firewall or Wi-Fi dependencies |

> **In short:** MotionArcade brings the magic of living-room camera motion gaming to any household or classroom without dedicated hardware barriers.

---

## Core Principles

- **Offline-First:** All core games, models, audio, and starter packs ship bundled. Downloadable packs persist locally. Zero internet required to play.
- **Hardware-Light:** Runs on standard laptops, mini PCs, smart TV boxes, or in a web browser using a basic webcam.
- **Privacy-by-Design:** Video frames are processed strictly on-device in memory and discarded immediately. Only 17 2D keypoint coordinates are processed.
- **Fun First, Learning Second:** Mechanics must be genuinely engaging in pure "Play Mode" before educational content is introduced.
- **100% Free for Kids & Schools:** Zero paywalls, locked content packs, or subscriptions for children's learning.
- **Zero-Friction & Kid-Operated:** Full gesture-driven menus allow a 5-year-old to launch and play without touching a keyboard or mouse.
- **Full Household & Classroom Utility:** Scales from solo kids learning, to full-classroom active participation, up to parent-vs-kid co-op fitness challenges.
- **Kid-Safe:** No advertising, no social chat, no in-app purchases, and local-only profiles.

---

## Documentation Index

Detailed specifications and technical designs have been split into modular documents:

- 📐 **[Technical Architecture & Pipeline](docs/architecture.md)** — Threading model, latency budget, hardware setup, and pose keypoint reference.
- 🎮 **[Game Design & Ergonomics](docs/game-design.md)** — Latency compensation, fatigue prevention, age bands, and tracking recovery.
- 📚 **[Learning System & Content Packs](docs/learning-system.md)** — TOML lesson pack format, Leitner box adaptive engine, and offline distribution.
- 🏫 **[Classroom Mode, Fitness & Roadmap](docs/classroom-and-fitness.md)** — School deployment, adult workout modes, and phase-by-phase roadmap.
- 💡 **[Sustainability & Business Model](docs/sustainability-and-business.md)** — Why all learning & schools are 100% free, monetizing adult fitness & plug-and-play TV hardware.

---

## Included Games & Themes

Mechanics are cleanly decoupled from visual themes. The platform spans multiple age-tailored worlds:

| Game | Theme & Target | Play Mechanic | Learning Integration | Status |
|---|---|---|---|---|
| **Jungle Jump (Banana Grab)** | 🐒 **Jungle Animals (Ages 4–7)** | Reach up to snatch falling bananas; duck under vines | Grab the bunch with the right number/letter to feed the hungry gorilla | **Phase 1 Target** |
| **Deflecto-Bot** | 🤖 **Cyber Arena (Ages 7–10)** | Extend hands to deflect incoming plasma orbs | Deflect orbs carrying the correct answer; let wrong answers pass | **Phase 0 Target** |
| **River Hop** | 🐸 **River Adventure (Ages 6–10)** | Lean & side-step onto floating river logs | Hop onto logs displaying the correct math solution | Planned |
| **Storm Surge** | 🥋 **Ninja Temple (Ages 8–12)** | Match body pose to fit through approaching wall cutouts | Strike letter/number shapes or lean into the correct answer slot | Planned |
| **Cargo Catch** | 📦 **Co-op Factory (Ages 7+)** | 2 players balance a platform with raised hands | Catch items matching specific categories (even numbers, vowels) | Planned |

---

## Quick Start

### Prerequisites
- Rust stable (`rustup`)
- Standard USB/integrated webcam
- Pose model (`.onnx`) placed in `assets/models/`

### Running the App
```bash
# Run with camera
cargo run --release

# Development flags
cargo run --release -- --game deflecto-bot  # Launch directly into game
cargo run --release -- --debug-skeleton     # Display tracking overlay
cargo run --release -- --mock-input         # Replay recorded poses without webcam
cargo run --release -- --validate-content   # Check TOML packs for schema errors
```

---

## Project Structure

```
motionarcade/
├── Cargo.toml                 # Workspace root
├── content/                   # Lesson packs (TOML + assets)
│   ├── bundled/               # Ships with the app (always offline)
│   └── downloaded/            # Locally cached downloaded packs
├── assets/
│   ├── models/                # ONNX pose models
│   ├── audio/                 # Sound effects & spoken audio
│   └── recordings/            # Mock pose recordings for testing
├── crates/
│   ├── core/                  # Keypoint math, smoothing, Game trait
│   ├── input/                 # Camera capture (nokhwa) & inference (ort)
│   ├── learning/              # Spaced repetition & pack loaders
│   ├── app/                   # Game registry, HUD, and renderer
│   └── games/                 # Self-contained game crates
└── docs/                      # Technical and design documentation
```

---

## Adding a Built-In Game

1. **Create the crate:**
   ```bash
   cargo new crates/games/my_game --lib
   ```
2. **Implement the `Game` trait:**
   ```rust
   use core::game::{Game, GameContext, GameStatus};
   use core::pose::PlayerInput;

   pub struct MyGame { score: u32 }

   impl Game for MyGame {
       fn id(&self) -> &'static str { "my-game" }
       fn name(&self) -> &'static str { "My Game" }

       fn update(&mut self, dt: f32, input: &PlayerInput, ctx: &mut GameContext) -> GameStatus {
           if let Some(player) = input.players.first() {
               if player.right_wrist().y < player.nose().y {
                   self.score += 1;
               }
           }
           GameStatus::Running
       }

       fn draw(&self, ctx: &mut GameContext) {
           ctx.canvas.text(&format!("Score: {}", self.score), 40.0, 60.0);
       }
   }
   ```
3. **Register in `crates/app/src/registry.rs`:**
   ```rust
   pub fn all_games() -> Vec<Box<dyn Game>> {
       vec![
           Box::new(deflecto_bot::DeflectoBot::new()),
           Box::new(my_game::MyGame::new()),
       ]
   }
   ```

---

## License

Dual-licensed under MIT or Apache-2.0.
