//! Architecture tests that verify dependency direction between workspace crates.
//!
//! The intended dependency graph:
//!   paw-proto   -- leaf, no internal deps
//!   paw-crypto  -- leaf, no internal deps
//!   paw-core    -- depends on paw-proto only
//!   paw-server  -- depends on paw-proto only

use std::fs;
use std::path::{Path, PathBuf};

/// Return the workspace root (one level up from paw-server).
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR has no parent")
        .to_path_buf()
}

/// Extract the `[dependencies]` section from a Cargo.toml string.
///
/// Returns everything from the first `[dependencies]` header (including
/// `[dependencies.foo]` sub-tables) up to the next unrelated section header,
/// or end of file.
fn extract_dependencies_section(cargo_toml: &str) -> String {
    let mut in_deps = false;
    let mut section = String::new();

    for line in cargo_toml.lines() {
        let trimmed = line.trim();

        if trimmed == "[dependencies]" {
            in_deps = true;
            continue;
        }

        // `[dependencies.foo]` sub-tables are still part of deps.
        if trimmed.starts_with("[dependencies.") {
            in_deps = true;
            section.push_str(line);
            section.push('\n');
            continue;
        }

        // Any other section header ends the dependencies block.
        if trimmed.starts_with('[') && in_deps {
            in_deps = false;
            continue;
        }

        if in_deps {
            section.push_str(line);
            section.push('\n');
        }
    }

    section
}

fn read_crate_cargo_toml(crate_dir: &str) -> String {
    let path = workspace_root().join(crate_dir).join("Cargo.toml");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
}

// ---------------------------------------------------------------------------
// paw-proto: leaf crate, no internal workspace dependencies
// ---------------------------------------------------------------------------

#[test]
fn paw_proto_has_no_internal_deps() {
    let deps = extract_dependencies_section(&read_crate_cargo_toml("paw-proto"));
    assert!(
        !deps.contains("paw-server"),
        "paw-proto must not depend on paw-server"
    );
    assert!(
        !deps.contains("paw-core"),
        "paw-proto must not depend on paw-core"
    );
    assert!(
        !deps.contains("paw-crypto"),
        "paw-proto must not depend on paw-crypto"
    );
}

// ---------------------------------------------------------------------------
// paw-crypto: leaf crate, no internal workspace dependencies
// ---------------------------------------------------------------------------

#[test]
fn paw_crypto_has_no_internal_deps() {
    let deps = extract_dependencies_section(&read_crate_cargo_toml("paw-crypto"));
    assert!(
        !deps.contains("paw-server"),
        "paw-crypto must not depend on paw-server"
    );
    assert!(
        !deps.contains("paw-core"),
        "paw-crypto must not depend on paw-core"
    );
    assert!(
        !deps.contains("paw-proto"),
        "paw-crypto must not depend on paw-proto"
    );
}

// ---------------------------------------------------------------------------
// paw-core: depends on paw-proto only
// ---------------------------------------------------------------------------

#[test]
fn paw_core_depends_on_paw_proto_only() {
    let deps = extract_dependencies_section(&read_crate_cargo_toml("paw-core"));
    assert!(
        deps.contains("paw-proto"),
        "paw-core should depend on paw-proto"
    );
    assert!(
        !deps.contains("paw-server"),
        "paw-core must not depend on paw-server"
    );
    assert!(
        !deps.contains("paw-crypto"),
        "paw-core must not depend on paw-crypto"
    );
}

// ---------------------------------------------------------------------------
// paw-server: depends on paw-proto only
// ---------------------------------------------------------------------------

#[test]
fn paw_server_depends_on_paw_proto_only() {
    let deps = extract_dependencies_section(&read_crate_cargo_toml("paw-server"));
    assert!(
        deps.contains("paw-proto"),
        "paw-server should depend on paw-proto"
    );
    assert!(
        !deps.contains("paw-core"),
        "paw-server must not depend on paw-core"
    );
    assert!(
        !deps.contains("paw-crypto"),
        "paw-server must not depend on paw-crypto"
    );
}

// ---------------------------------------------------------------------------
// No circular dependencies: if A depends on B, B must not depend on A
// ---------------------------------------------------------------------------

#[test]
fn no_circular_dependencies() {
    let core_deps = extract_dependencies_section(&read_crate_cargo_toml("paw-core"));
    let server_deps = extract_dependencies_section(&read_crate_cargo_toml("paw-server"));
    let proto_deps = extract_dependencies_section(&read_crate_cargo_toml("paw-proto"));
    let crypto_deps = extract_dependencies_section(&read_crate_cargo_toml("paw-crypto"));

    // paw-core -> paw-proto, so paw-proto must not -> paw-core
    if core_deps.contains("paw-proto") {
        assert!(
            !proto_deps.contains("paw-core"),
            "Circular dependency: paw-core -> paw-proto and paw-proto -> paw-core"
        );
    }

    // paw-server -> paw-proto, so paw-proto must not -> paw-server
    if server_deps.contains("paw-proto") {
        assert!(
            !proto_deps.contains("paw-server"),
            "Circular dependency: paw-server -> paw-proto and paw-proto -> paw-server"
        );
    }

    // paw-core and paw-server must not depend on each other
    if core_deps.contains("paw-server") {
        assert!(
            !server_deps.contains("paw-core"),
            "Circular dependency: paw-core -> paw-server and paw-server -> paw-core"
        );
    }

    // crypto must not form cycles with anyone
    if crypto_deps.contains("paw-proto") {
        assert!(
            !proto_deps.contains("paw-crypto"),
            "Circular dependency: paw-crypto -> paw-proto and paw-proto -> paw-crypto"
        );
    }
    if crypto_deps.contains("paw-core") {
        assert!(
            !core_deps.contains("paw-crypto"),
            "Circular dependency: paw-crypto -> paw-core and paw-core -> paw-crypto"
        );
    }
    if crypto_deps.contains("paw-server") {
        assert!(
            !server_deps.contains("paw-crypto"),
            "Circular dependency: paw-crypto -> paw-server and paw-server -> paw-crypto"
        );
    }
}
