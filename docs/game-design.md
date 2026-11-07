# Game Design, Mechanics & Child Ergonomics

This document details game mechanics, cognitive load balancing, physical safety, and ergonomics for children and classroom environments.

---

## 1. Game Design Philosophy

> **The Golden Rule:** A game must be fun with all educational questions turned off. If kids won't play it in pure "Play Mode", adding math will not fix it.

### Design for the Body, Not the Thumbs

| Traditional Input | MotionArcade Equivalent | Consideration |
|---|---|---|
| Press button to jump | Physically jump or raise arms | High energy — limit to short bursts |
| Joystick left/right | Lean torso or step laterally | Natural, low fatigue, clear camera tracking |
| Button to select | Reach into target zone | Forgiving hit zones (±15 cm minimum) |
| Hold button to charge | Hold pose (squat, T-pose) | Cap at 2–3 seconds; tiring after 5s |

### The "Juice" Factor: Replacing Physical Controller Haptics

Without physical gamepads, players have zero mechanical tactile feedback. Game "juice" creates synthetic haptics:
- **Audio Clack & Bass Impact:** Striking or deflecting an orb triggers an immediate, punchy sound wave timed exactly to keypoint impact.
- **Visual Impact Sparks:** High-speed particle bursts, comic-style hit lines, and micro screen-shake centered at the deflection coordinate.
- **Stylized Player Aura / Silhouette:** Rather than raw webcam footage (messy rooms, self-consciousness) or sterile stick figures, render a colorful **glowing superhero aura / neon silhouette** over the player. Kids love seeing their body transformed into an active avatar.

### Zero-Friction Gesture Menus (Kid-Operated)

Parents should not have to run across the room with a mouse to select levels or start new rounds:
- **Hover-to-Select Dwell Buttons:** Holding a hand in a menu orb for 1.0 second triggers selection with a radial fill animation.
- **Universal "Jump to Start" / "Wave to Play":** Active body gestures confirm readiness, enabling children as young as 4 to play independently.

### Fatigue Management ("Gorilla Arm")

Kids and adults fatigue quickly when maintaining raised arm gestures.
- **Intervals:** 15–30 seconds of high activity followed by 5–10 second resting periods (score celebration, story beat).
- **Session Limits:** Automatic prompt to take a break after 8–10 minutes.
- **Muscle Alternation:** Avoid continuous shoulder-height poses across successive gameplay tasks.

### Cognitive Load vs. Motor Load

Children struggle to process complex textual questions while managing balance and physical movement.
- Prompts must be parsed in **< 0.5 seconds** (colors, numbers, icons, shapes).
- **Spoken audio** is mandatory for pre-readers (ages 4–6).
- Questions should *be* the obstacles/mechanics, never modal popups that pause the game.

### Handling Tracking Loss

When lighting changes, occlusion happens, or a child twists sideways:
1. **Never penalize:** Freeze state without registering a fail.
2. **Graceful UI:** Display an animated prompt: *"I lost you! Wave your hands!"*
3. **Player Locking:** Lock onto initial players by spatial bounding box; ignore pets or passing siblings.

---

## 2. Age Bands & Ergonomics

| Age Band | Choices | Prompts | Pacing | Target Zones | Session Length |
|---|---|---|---|---|---|
| **4–6** | 2 | Audio + large icons | Slow, no timer | Huge (±20 cm) | 5 min |
| **7–9** | 3–4 | Short text + audio | Gentle timer | Large (±15 cm) | 8 min |
| **10–12** | 4 | Text | Timed / competitive | Standard (±10 cm) | 10 min |

### Accessibility (Mandatory)

- **Seated Mode:** Complete gameplay supported through upper-body and head movements only.
- **Single-Arm Mode:** Playable using one arm for children with temporary injuries or limb differences.
- **Reduced Motion:** Calmer visual pacing, no rapid flashing lights.

