fn main() {
    uniffi::generate_scaffolding("src/paw_core.udl")
        .expect("failed to generate paw-core scaffolding");
}
