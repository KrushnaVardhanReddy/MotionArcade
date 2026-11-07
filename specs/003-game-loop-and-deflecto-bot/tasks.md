# Tasks: Spec 003 — Game Loop, Deflecto-Bot & Synthetic Haptics

**Spec Reference:** [`specs/003-game-loop-and-deflecto-bot/spec.md`](spec.md)  
**Quality Standards:** Locked 60 fps rendering, hands-free kid gesture menus, audio/visual juice.

---

## Task Breakdown

- [ ] **Task 3.1: Macroquad Game Loop & Keypoint Interpolation**
  - Implement 60 fps main render loop in `crates/app/src/main.rs`.
  - Interpolate keypoints linearly between 15–30 fps inference ticks for buttery movement.

- [ ] **Task 3.2: Deflecto-Bot Quadrant Mechanics**
  - Implement 4-quadrant hitzones in `crates/games/deflecto_bot/src/lib.rs`.
  - Implement 0.15s dwell verification for hand collision detection.

- [ ] **Task 3.3: Synthetic Haptics ("Juice")**
  - Implement particle firework explosions at impact coordinate.
  - Implement camera screen-punch effect.
  - Wire positional audio chime triggers via `rodio`.

- [ ] **Task 3.4: Player Neon Silhouette & Floor Anchor**
  - Render glowing stylized player aura/silhouette over tracked keypoints.
  - Render floor boundary calibration ring on screen.

- [ ] **Task 3.5: Hands-Free Gesture Menus**
  - Implement hover-to-select dwell button in menu orb.
  - Wave to start round without mouse/keyboard.

- [ ] **Task 3.6: Jungle Banana Grab Theme Skin**
  - Implement monkey/banana sprite asset variant over the quadrant reach engine.
  - Add cartoon chomp and tropical leaf particle effects.

