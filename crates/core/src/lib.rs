//! MotionArcade Core Domain Engine
//!
//! Pure, deterministic domain logic and hexagonal port definitions for MotionArcade:
//! - 17-keypoint COCO skeleton model (`pose`)
//! - One Euro adaptive low-pass smoothing filter (`smoothing`)
//! - Spatial floor boundary cylinder filter (`boundary`)
//! - Inbound stream ports and mock file adapter (`ports`)
//!
//! Architectural guarantees:
//! - Zero camera hardware, windowing, or graphics framework dependencies.
//! - Safe error handling: zero `.unwrap()` or `.expect()` in non-test code.
//! - Thread-safe models: all types implement `Send + Sync + 'static`.

pub mod boundary;
pub mod ports;
pub mod pose;
pub mod smoothing;

pub use boundary::{FloorBoundary, FloorCylinderBoundary};
pub use ports::input::{PoseStreamPort, TrackingError};
pub use ports::mock::MockFilePoseStreamAdapter;
pub use pose::{Keypoint, PlayerInput, Skeleton};
pub use smoothing::{OneEuroFilter, OneEuroFilter2D, OneEuroFilterConfig, SkeletonSmoother};

// Re-export standard core modules to prevent doctest name shadowing issues when crate is named `core`.
#[doc(hidden)]
pub use std::{convert, fmt, option, write};

/// Domain-level error encompassing all core engine error conditions.
#[derive(thiserror::Error, Debug)]
pub enum CoreError {
    #[error("Tracking pipeline error: {0}")]
    Tracking(#[from] TrackingError),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Filter error: {0}")]
    Filter(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
