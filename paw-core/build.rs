fn main() {
    uniffi::generate_scaffolding("uniffi/paw_core.udl")
        .expect("failed to generate paw-core scaffolding");
}
