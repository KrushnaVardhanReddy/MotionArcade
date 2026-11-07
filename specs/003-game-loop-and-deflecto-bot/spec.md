# Spec 003: Game Loop, Deflecto-Bot & Synthetic Haptics

## Status
Draft / In Design

---

## 1. Goal
Implement the flagship mini-game **Deflecto-Bot** with 60 fps rendering, gesture menus, flying plasma orbs, collision hitboxes, and audio-visual juice.

---

## 2. Core Mechanics
1. **Target Zones:** 4 quadrants (Top-Left, Top-Right, Bottom-Left, Bottom-Right).
2. **Player Interaction:** Hand keypoint enters quadrant holding position for 0.15s (dwell confirmation).
3. **Synthetic Haptics:** Particle sparks, camera micro-shake, and positional chord sound on impact.
4. **Kid Gesture Menu:** Wave hand in menu orb to launch game without keyboard/mouse.

---

## 3. Acceptance Criteria
- [ ] Runs at 60 fps locked.
- [ ] Works with `--mock-input` recorded sequence.
- [ ] Displays neon silhouette overlay mirroring player posture.
