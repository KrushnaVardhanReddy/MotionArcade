# Spec 006: Jungle Jump — Banana Grab Animal Theme Game (Ages 4–7)

## Status
Approved / Ready for Parallel Implementation

---

## 1. Context & Motivation
While Deflecto-Bot focuses on older kids (7–10) with sci-fi cyber orbs, **Jungle Jump (Banana Grab)** is designed specifically for young children (ages 4–7).
It features vertical falling banana bunches from the jungle canopy, reaching gestures to feed a hungry gorilla, swinging vines to duck under, cartoon chomp sounds, and tropical leaf particle bursts.

---

## 2. Zero-Conflict Architecture
- Entirely isolated in its own new crate: `crates/games/banana_grab/`.
- Depends only on `crates/core` (implements the `core::game::Game` trait).
- Zero shared file edits with the active Deflecto-Bot subagent.

---

## 3. Core Mechanics
1. **Vertical Falling Fruit:** Bananas, mangoes, and coconuts fall vertically from the top of the screen (`y: 0.0 -> 1.0`).
2. **High Reach Collision:** When a child raises their hand above shoulder/nose height (`skeleton.right_wrist().y < skeleton.nose().y`), any fruit in that column is grabbed.
3. **Cartoon Particle FX:** Grabbing fruit spawns green jungle leaf particles and increments the banana counter.
4. **Vine Ducking:** A swinging vine sweeps across the upper third; child physically ducks or squats to dodge.

---

## 4. Acceptance Criteria
- [ ] `BananaGrab` implements `core::game::Game`.
- [ ] Unit tests verify high-reach fruit collision, falling physics, and leaf particle decays.
- [ ] Zero `.unwrap()` in production code.
- [ ] Compiles and passes `cargo test -p banana_grab`.
