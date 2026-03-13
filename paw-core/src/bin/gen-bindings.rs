use camino::Utf8PathBuf;
use std::env;
use uniffi::{generate_bindings, KotlinBindingGenerator, SwiftBindingGenerator};

fn main() {
    let mut args = env::args().skip(1);
    let language = args
        .next()
        .expect("usage: cargo run -p paw-core --bin gen-bindings -- <kotlin|swift> <out-dir>");
    let out_dir = Utf8PathBuf::from(
        args.next()
            .expect("usage: cargo run -p paw-core --bin gen-bindings -- <kotlin|swift> <out-dir>"),
    );

    let udl = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("uniffi/paw_core.udl");

    match language.as_str() {
        "kotlin" => generate_bindings(
            &udl,
            None,
            KotlinBindingGenerator,
            Some(&out_dir),
            None,
            None,
            true,
        )
        .expect("failed to generate Kotlin bindings"),
        "swift" => generate_bindings(
            &udl,
            None,
            SwiftBindingGenerator,
            Some(&out_dir),
            None,
            None,
            true,
        )
        .expect("failed to generate Swift bindings"),
        _ => panic!("unsupported language: {language}"),
    }
}
