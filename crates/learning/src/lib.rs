//! MotionArcade Learning Engine
//!
//! Offline-first TOML content pack parsing, CSV import, Leitner spaced repetition,
//! and classroom session reports.
//!
//! Guarantees:
//! - Zero panics (`.unwrap()` / `.expect()`) in production code.
//! - 100% offline schema validation.
//! - Deterministic 5-box spaced repetition.
//! - Accurate progress tracking without camera latency penalties.

pub mod csv_importer;
pub mod leitner;
pub mod loader;
pub mod pack;
pub mod profile;

pub use csv_importer::{import_csv_file_to_toml_file, parse_csv_to_pack};
pub use leitner::{LeitnerBox, LeitnerProgress, LeitnerScheduler};
pub use loader::{load_pack_from_file, load_pack_from_str, validate_pack_file};
pub use pack::{ContentError, ContentPack, ContentPackHeader, LessonItem};
pub use profile::{ItemSummaryRow, SessionReport, StudentProfile};
