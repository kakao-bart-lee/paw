use camino::Utf8PathBuf;
use cargo_metadata::MetadataCommand;
use std::env;
use std::process::Command;
use uniffi::{
    generate_bindings_library_mode, CargoMetadataConfigSupplier, KotlinBindingGenerator,
    SwiftBindingGenerator,
};

fn ensure_library_exists(library_path: &Utf8PathBuf, profile: &str) {
    if library_path.exists() {
        return;
    }

    let mut cmd = Command::new("cargo");
    cmd.args(["build", "-p", env!("CARGO_PKG_NAME"), "--lib"]);
    if profile == "release" {
        cmd.arg("--release");
    }

    let status = cmd
        .status()
        .expect("failed to invoke cargo build for UniFFI library generation");
    assert!(
        status.success(),
        "cargo build for UniFFI library generation failed with status {status}"
    );
    assert!(
        library_path.exists(),
        "expected generated library at {library_path}, but it does not exist"
    );
}

fn main() {
    let mut args = env::args().skip(1);
    let language = args
        .next()
        .expect("usage: cargo run -p paw-core --bin gen-bindings -- <kotlin|swift> <out-dir>");
    let out_dir = Utf8PathBuf::from(
        args.next()
            .expect("usage: cargo run -p paw-core --bin gen-bindings -- <kotlin|swift> <out-dir>"),
    );
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .expect("failed to run cargo metadata");
    let config_supplier = CargoMetadataConfigSupplier::from(metadata.clone());
    let target_dir = Utf8PathBuf::from_path_buf(metadata.target_directory.into_std_path_buf())
        .expect("target directory must be valid utf-8");
    let profile = env::var("PAW_UNIFFI_PROFILE").unwrap_or_else(|_| "debug".to_string());
    let lib_ext = if cfg!(target_os = "macos") {
        "dylib"
    } else if cfg!(target_os = "windows") {
        "dll"
    } else {
        "so"
    };
    let library_path = target_dir.join(&profile).join(format!(
        "lib{}.{}",
        env!("CARGO_PKG_NAME").replace('-', "_"),
        lib_ext
    ));

    ensure_library_exists(&library_path, &profile);

    match language.as_str() {
        "kotlin" => {
            generate_bindings_library_mode(
                &library_path,
                None,
                &KotlinBindingGenerator,
                &config_supplier,
                None,
                &out_dir,
                true,
            )
            .expect("failed to generate Kotlin bindings");
        }
        "swift" => {
            generate_bindings_library_mode(
                &library_path,
                None,
                &SwiftBindingGenerator,
                &config_supplier,
                None,
                &out_dir,
                true,
            )
            .expect("failed to generate Swift bindings");
        }
        _ => panic!("unsupported language: {language}"),
    };
}
