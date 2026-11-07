# Contract 008: Game Audio Wiring

**Contract ID:** `CONTRACT-AUDIO-008`  
**Crates:** `crates/games/deflecto_bot`, `crates/games/banana_grab`  
**Guarantees:** Zero `.unwrap()`, non-blocking, headless CI safe.

---

```rust
use audio::{AudioMixer, SoundEffect};
use std::sync::Arc;

impl DeflectoBot {
    pub fn with_audio(mut self, mixer: Arc<AudioMixer>) -> Self;
}

impl BananaGrab {
    pub fn with_audio(mut self, mixer: Arc<AudioMixer>) -> Self;
}
```
