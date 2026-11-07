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

## 2. Age Bands, Ergonomics & Themes

MotionArcade supports distinct visual and narrative themes tailored to developmental stages. Because mechanics are decoupled from visuals, the underlying hit detection engine remains identical while art styles adapt:

| Age Band | Primary Themes | Target Tone & Humor | Preferred Mechanics |
|---|---|---|---|
| **4–6 (Early Years)** | **Jungle Animals, Dinosaurs, Ocean Friends** | Funny animal sounds, bouncing fruit, cartoon splats | High reaches (banana grabs), big squats (ducking vines), bubble pops |
| **7–9 (Middle Arcade)** | **Robots, Space Explorers, Superhero Auras** | Energy blasts, laser shields, power-up streaks | Fast quadrant reaches (Deflecto-Bot), platform hopping (River Hop) |
| **10–12 (Tween/Classroom)** | **Extreme Sports, Ninja Obstacles, Team Relay** | Rhythm combos, high score badges, team challenges | Pose matching (Storm Surge), co-op cargo balancing |

---

## 3. Flagship Game Lineup

Every game works in **Play Mode** (pure arcade fun) or **Learn Mode** (integrated with a TOML content pack).

### A. Jungle Jump: Banana Grab 🐒🍌 (Ages 4–7)
- **Theme:** The child plays as a nimble monkey in a vibrant jungle canopy!
- **Play Mode:** Bunches of bananas and golden mangoes fall from treetops. Reach high and stretch laterally to catch delicious fruit while ducking under swinging vines.
- **Learn Mode:** 
  - *Math:* The hungry gorilla asks for `6` bananas. Batches of `2`, `3`, and `4` bananas fall—grab the right bunch!
  - *Phonics:* Fruits carry letters. *"Grab the banana that starts with the sound 'B'!"*
- **The Juice:** Hilarious cartoon chomp sounds, tropical leaf confetti explosions, and celebratory monkey backflips on correct grabs.

### B. Deflecto-Bot 🤖⚡ (Ages 7–10)
- **Theme:** Sci-fi cyber arena with a floating robot assistant.
- **Play Mode:** High-speed plasma orbs fly toward the player; deflect all incoming projectiles across 4 quadrants.
- **Learn Mode:** Orbs carry answer options. Deflect the correct answer to fuel your laser shield; let incorrect distractors safely pass.
- **The Juice:** Heavy bass impact chime, golden starburst fireworks, and a glowing neon superhero avatar overlay.

### C. Storm Surge 🌊🥋 (Ages 8–12)
- **Theme:** Ninja training temple facing moving mystical obstacle walls.
- **Play Mode:** Moving walls with geometric human cutouts approach; match your body pose (T-pose, squat, high knee) to slip through.
- **Learn Mode:** Wall cutouts form numbers, letters, or answer slots. Pose into the slot matching the prompt.

### D. River Hop 🐸🪵 (Ages 6–10)
- **Theme:** River adventure with hopping stones and floating logs.
- **Play Mode:** Step side-to-side and lean across river platforms to escape rising water.
- **Learn Mode:** Each floating log carries an answer. Hop onto the correct solution to cross safely.

---

## 4. Age Bands & Ergonomics

| Age Band | Choices | Prompts | Pacing | Target Zones | Session Length |
|---|---|---|---|---|---|
| **4–6** | 2 | Audio + large icons | Slow, no timer | Huge (±20 cm) | 5 min |
| **7–9** | 3–4 | Short text + audio | Gentle timer | Large (±15 cm) | 8 min |
| **10–12** | 4 | Text | Timed / competitive | Standard (±10 cm) | 10 min |

### Accessibility (Mandatory)

- **Seated Mode:** Complete gameplay supported through upper-body and head movements only.
- **Single-Arm Mode:** Playable using one arm for children with temporary injuries or limb differences.
- **Reduced Motion:** Calmer visual pacing, no rapid flashing lights.

