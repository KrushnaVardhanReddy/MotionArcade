//! Integration tests validating all official bundled content packs.
//!
//! Validates schema integrity, distractor constraints, unique IDs,
//! and TOML round-tripping for literacy, math, and science packs.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use learning::loader::load_pack_from_file;
use learning::pack::ContentPack;

/// Helper to resolve content pack paths from various working directory contexts.
fn resolve_pack_path(relative_path: &str) -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest_dir.join("../../content/bundled").join(relative_path),
        manifest_dir.join("../content/bundled").join(relative_path),
        manifest_dir.join("content/bundled").join(relative_path),
        Path::new("content/bundled").join(relative_path),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    panic!(
        "Failed to resolve bundled pack path '{}'. Checked candidates: {:?}",
        relative_path, candidates
    );
}

/// Helper that verifies common schema guarantees for any loaded pack.
fn verify_pack_invariants(pack: &ContentPack, expected_min_items: usize) {
    // 1. Pack-level validate() must succeed
    pack.validate().expect("pack.validate() failed");

    // 2. Offline flag must be true for all bundled packs
    assert!(pack.pack.offline, "Bundled pack must be marked offline");

    // 3. Header fields must be trimmed and non-empty
    assert!(!pack.pack.id.trim().is_empty());
    assert!(!pack.pack.title.trim().is_empty());
    assert!(!pack.pack.subject.trim().is_empty());
    assert!(!pack.pack.age_band.trim().is_empty());

    // 4. Minimum items requirement
    assert!(
        pack.items.len() >= expected_min_items,
        "Pack {} has {} items, expected at least {}",
        pack.pack.id,
        pack.items.len(),
        expected_min_items
    );

    // 5. Item integrity checks
    let mut seen_ids = HashSet::new();
    for item in &pack.items {
        // Unique ID
        assert!(
            seen_ids.insert(item.id.clone()),
            "Duplicate item id '{}' in pack '{}'",
            item.id,
            pack.pack.id
        );

        // Non-empty fields
        assert!(!item.prompt.trim().is_empty());
        assert!(!item.answer.trim().is_empty());
        assert!(!item.skill.trim().is_empty());

        // At least 3 distractors
        assert!(
            item.distractors.len() >= 3,
            "Item '{}' in pack '{}' has {} distractors, expected at least 3",
            item.id,
            pack.pack.id,
            item.distractors.len()
        );

        // Distractor uniqueness & answer distinctness
        let mut seen_distractors = HashSet::new();
        for d in &item.distractors {
            let trimmed = d.trim();
            assert!(!trimmed.is_empty(), "Empty distractor in item '{}'", item.id);
            assert!(
                !trimmed.eq_ignore_ascii_case(item.answer.trim()),
                "Distractor '{}' matches answer '{}' in item '{}'",
                d,
                item.answer,
                item.id
            );
            assert!(
                seen_distractors.insert(trimmed.to_lowercase()),
                "Duplicate distractor '{}' in item '{}'",
                d,
                item.id
            );
        }
    }

    // 6. TOML round-trip test
    let serialized = pack.to_toml_string().expect("Failed to serialize pack");
    let reparsed = ContentPack::from_toml_str(&serialized).expect("Failed to reparsed serialized pack");
    assert_eq!(*pack, reparsed, "Round-trip TOML serialization mismatch for pack '{}'", pack.pack.id);
}

#[test]
fn test_bundled_math_number_facts_k2() {
    let path = resolve_pack_path("math/number_facts_k2.toml");
    let pack = load_pack_from_file(&path).expect("Failed to load number_facts_k2.toml");

    assert_eq!(pack.pack.id, "math-number-facts-k2");
    assert_eq!(pack.pack.subject, "math");
    assert_eq!(pack.pack.age_band, "5-8");
    assert!(!pack.pack.reading_required);
    assert_eq!(pack.items.len(), 10);

    verify_pack_invariants(&pack, 10);
}

#[test]
fn test_bundled_literacy_letter_sounds_k1() {
    let path = resolve_pack_path("literacy/letter_sounds_k1.toml");
    let pack = load_pack_from_file(&path).expect("Failed to load letter_sounds_k1.toml");

    assert_eq!(pack.pack.id, "literacy-letter-sounds-k1");
    assert_eq!(pack.pack.subject, "literacy");
    assert_eq!(pack.pack.age_band, "4-6");
    assert!(!pack.pack.reading_required);
    assert_eq!(pack.items.len(), 26);

    verify_pack_invariants(&pack, 26);
}

#[test]
fn test_bundled_math_multiplication_basics() {
    let path = resolve_pack_path("math/multiplication_basics.toml");
    let pack = load_pack_from_file(&path).expect("Failed to load multiplication_basics.toml");

    assert_eq!(pack.pack.id, "math-multiplication-basics");
    assert_eq!(pack.pack.subject, "math");
    assert_eq!(pack.pack.age_band, "7-9");
    assert!(!pack.pack.reading_required);
    assert_eq!(pack.items.len(), 20);

    verify_pack_invariants(&pack, 20);
}

#[test]
fn test_bundled_science_animal_habitats() {
    let path = resolve_pack_path("science/animal_habitats.toml");
    let pack = load_pack_from_file(&path).expect("Failed to load animal_habitats.toml");

    assert_eq!(pack.pack.id, "science-animal-habitats");
    assert_eq!(pack.pack.subject, "science");
    assert_eq!(pack.pack.age_band, "5-8");
    assert!(pack.pack.reading_required);
    assert_eq!(pack.items.len(), 15);

    verify_pack_invariants(&pack, 15);
}

#[test]
fn test_all_four_bundled_packs_load_and_validate() {
    let pack_files = [
        ("math/number_facts_k2.toml", 10),
        ("literacy/letter_sounds_k1.toml", 26),
        ("math/multiplication_basics.toml", 20),
        ("science/animal_habitats.toml", 15),
    ];

    for (rel_path, expected_count) in &pack_files {
        let path = resolve_pack_path(rel_path);
        let pack = load_pack_from_file(&path)
            .unwrap_or_else(|e| panic!("Failed loading {}: {:?}", rel_path, e));
        assert_eq!(
            pack.items.len(),
            *expected_count,
            "Item count mismatch for pack {}",
            rel_path
        );
        verify_pack_invariants(&pack, *expected_count);
    }
}
