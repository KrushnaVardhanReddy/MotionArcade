# Spec 003: Game Loop, Deflecto-Bot & Jungle Banana Grab

## Status
Draft / In Design

---

## 1. Goal
Implement the core 60 fps native GPU rendering loop with custom shaders, kid gesture menus, and two flagship theme implementations sharing the same underlying quadrant collision engine:
1. **Deflecto-Bot (Cyber Robot & Orbs)** — targeted at ages 7–10.
2. **Jungle Jump / Banana Grab (Monkey & Tropical Fruits)** — targeted at ages 4–7.

---

## 2. Rendering & UI Architecture (Native GPU Engine)

To match the console-level feel of products like Nex Playground, the rendering loop bypasses browser webviews in favor of a native GPU rendering pipeline using `macroquad` / `miniquad` / `wgpu`:

```
 Camera Buffer (nokhwa) ──► Direct Texture Upload (VRAM)
                                  │
                                  ▼
      Skeletal Keypoints  ──► Custom Fragment Shader (Neon Aura & Silhouette)
                                  │
                                  ▼
      Orbs & HUD          ──► Instanced Sprites + GPU Particle Explosion
                                  │
                                  ▼
                       Locked 60 FPS Presentation
```

1. **Direct Video Texture Streaming:** The webcam frame is uploaded straight into GPU VRAM in < 2ms without CPU memory copies.
2. **Neon Superhero Silhouette Shader:** A custom GLSL/WGSL fragment shader draws a luminous, vibrant outline/silhouette directly over the player's tracked coordinates.
3. **Instanced Particle Fireworks:** Deflection impact triggers 50–100 GPU-instanced star particles with color shifts and micro-shake.
4. **Kid-Friendly Native Menus:** Dwell-timer radial buttons drawn directly on-screen with zero external webview overhead.

---

## 3. Core Mechanics
1. **Target Zones:** 4 quadrants (Top-Left, Top-Right, Bottom-Left, Bottom-Right).
2. **Player Interaction:** Hand keypoint enters quadrant holding position for 0.15s (dwell confirmation).
3. **Synthetic Haptics:** Particle sparks, camera micro-shake, and positional chord sound on impact.
4. **Kid Gesture Menu:** Wave hand in menu orb to launch game without keyboard/mouse.

---

## 3. Acceptance Criteria
- [ ] Runs at 60 fps locked.
- [ ] Works with `--mock-input` recorded sequence.
- [ ] Displays neon silhouette overlay mirroring player posture.

