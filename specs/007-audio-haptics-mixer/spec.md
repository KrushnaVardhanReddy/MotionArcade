# Spec 007: Positional Audio & Synthetic Haptic Sound Mixer

## Status
Approved / Ready for Parallel Implementation

---

## 1. Context & Motivation
To make deflecting orbs and catching bananas feel impactful without physical controller vibration, MotionArcade needs an asynchronous sound synthesizer and sample player (`rodio`).
It plays positional stereo chimes (left hand deflection plays on left speaker, right on right), cartoon chomps, and celebratory fanfare.

---

## 2. Zero-Conflict Crate Isolation
- Lives entirely in `crates/audio/`.
- No overlap with active agents in `crates/games/deflecto_bot`, `crates/games/banana_grab`, or `crates/learning`.
- Can run headlessly in CI (mocks audio device if ALSA/PulseAudio/JACK is absent).

---

## 3. Acceptance Criteria
- [ ] `SoundEffect` enum: `DeflectionChime { pan: f32 }`, `BananaChomp`, `CorrectFanfare`, `GentleWobble`.
- [ ] Non-blocking channel trigger: game loops never block waiting for audio buffers.
- [ ] Headless CI verification: doesn't panic if no audio hardware device is found.
- [ ] Exactly zero `.unwrap()` in production code.
