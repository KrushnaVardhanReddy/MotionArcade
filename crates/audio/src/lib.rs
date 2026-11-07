//! MotionArcade Audio Engine
//!
//! Provides asynchronous sound playback and procedural tone synthesis:
//! - Positional stereo deflection chimes with dynamic left-right panning
//! - Cartoon banana chomp sound effects
//! - Triumphant celebratory fanfare arpeggios
//! - Playful frequency-modulated gentle wobble
//!
//! # Architectural Guarantees
//! - Zero `.unwrap()` in production code.
//! - Non-blocking channel message dispatch: game loops never block.
//! - Robust headless fallback: falls back gracefully without panicking if no audio hardware is present.

pub mod mixer;
pub mod synth;

pub use mixer::{AudioCommand, AudioMixer};
pub use synth::{
    generate_banana_chomp, generate_correct_fanfare, generate_deflect_chime,
    generate_gentle_wobble, generate_samples,
};

/// Sound effect catalog for MotionArcade gameplay and educational interactions.
#[derive(Debug, Clone, PartialEq)]
pub enum SoundEffect {
    /// Positional deflection chime with stereo pan (-1.0 = hard left, 0.0 = center, +1.0 = hard right).
    DeflectChime { pan: f32 },
    /// Alias for `DeflectChime` for specification compatibility.
    DeflectionChime { pan: f32 },
    /// Cartoon chomp sound for catching falling fruits or feeding characters.
    BananaChomp,
    /// Celebratory ascending fanfare arpeggio for correct answers and victories.
    CorrectFanfare,
    /// Playful frequency-modulated wobble for boundary detection and ducking.
    GentleWobble,
}

impl SoundEffect {
    /// Helper to construct a deflection chime with stereo panning.
    pub fn deflect(pan: f32) -> Self {
        Self::DeflectChime { pan }
    }

    /// Human-readable name for logging and telemetry.
    pub fn name(&self) -> &'static str {
        match self {
            Self::DeflectChime { .. } | Self::DeflectionChime { .. } => "DeflectChime",
            Self::BananaChomp => "BananaChomp",
            Self::CorrectFanfare => "CorrectFanfare",
            Self::GentleWobble => "GentleWobble",
        }
    }
}

/// Domain errors that can occur within the audio engine.
#[derive(thiserror::Error, Debug)]
pub enum AudioError {
    #[error("Audio output device initialization failed: {0}")]
    DeviceInitialization(String),

    #[error("Audio worker channel disconnected")]
    ChannelDisconnected,

    #[error("Audio worker thread spawn error: {0}")]
    ThreadSpawn(#[from] std::io::Error),
}
