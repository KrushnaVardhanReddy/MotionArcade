# Spec 009: Hands-Free Gesture Menu & Game Selector

## Status
Approved / Ready for Parallel Implementation

---

## 1. Context & Motivation
For a 5-year-old child at home, using a mouse or keyboard to choose games is high friction.
Spec 009 implements an interactive **Hands-Free Gesture Portal Menu** inside `crates/app`:
- Two large floating circular portals on screen:
  - Left Portal (Blue/Orange Jungle): **🐒 Jungle Banana Grab**
  - Right Portal (Cyan/Gold Cyber): **🤖 Deflecto-Bot Math**
- **Dwell Timer Activation:** When the child holds either hand inside a portal circle for **1.0 second**, a radial fill ring animates. Once full, the game transitions smoothly into the selected experience!
- An active game can be exited back to menu by holding both arms in a "T-Pose" or raising both hands for 3.0 seconds.

---

## 2. Zero-Conflict Crate Isolation
- Operates strictly in:
  - `crates/app/src/menu.rs` (Gesture menu state machine and dwell timers)
  - `crates/app/src/lib.rs` (Export menu module)
  - `crates/app/src/main.rs` (Switch between Menu, DeflectoBot, and BananaGrab)
  - `crates/app/tests/menu_test.rs`

---

## 3. Acceptance Criteria
- [ ] Dwell timer requires sustained hand presence inside portal radius for 1.0s.
- [ ] Moving hand away before 1.0s resets the dwell fill smoothly.
- [ ] Seamless transition to both `DeflectoBot` and `BananaGrab`.
- [ ] Exactly 0 `.unwrap()` in production code.

