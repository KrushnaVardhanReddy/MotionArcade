//! Pack loader and standalone content pack validator.
//!
//! Provides file-level loading and validation routines for offline content packs.

use std::path::Path;

use crate::pack::{ContentError, ContentPack};

/// Loads and validates a content pack from a TOML string.
pub fn load_pack_from_str(toml_str: &str) -> Result<ContentPack, ContentError> {
    ContentPack::from_toml_str(toml_str)
}

/// Loads and validates a content pack from a TOML file path.
pub fn load_pack_from_file(path: impl AsRef<Path>) -> Result<ContentPack, ContentError> {
    ContentPack::from_file(path)
}

/// Validates a content pack file on disk, including optional audio asset path checks.
pub fn validate_pack_file(
    pack_path: impl AsRef<Path>,
    assets_root: Option<impl AsRef<Path>>,
) -> Result<ContentPack, ContentError> {
    let pack = ContentPack::from_file(pack_path)?;
    if let Some(root) = assets_root {
        pack.validate_assets(root)?;
    }
    Ok(pack)
}
