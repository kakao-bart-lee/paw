use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;

const CIPHERSUITE: Ciphersuite = Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

pub fn create_credential(identity: &[u8]) -> Credential {
    BasicCredential::new(identity.to_vec()).into()
}

pub fn create_key_package(credential: &Credential) -> KeyPackage {
    let provider = OpenMlsRustCrypto::default();
    let signer = SignatureKeyPair::new(CIPHERSUITE.signature_algorithm())
        .expect("failed to generate signature key pair");
    signer
        .store(provider.storage())
        .expect("failed to store signature key pair");

    let credential_with_key = CredentialWithKey {
        credential: credential.clone(),
        signature_key: signer.to_public_vec().into(),
    };

    KeyPackage::builder()
        .key_package_extensions(Extensions::default())
        .build(CIPHERSUITE, &provider, &signer, credential_with_key)
        .expect("failed to build key package")
        .key_package()
        .clone()
}

pub fn create_group(creator_credential: &Credential) -> MlsGroup {
    let provider = OpenMlsRustCrypto::default();
    let signer = SignatureKeyPair::new(CIPHERSUITE.signature_algorithm())
        .expect("failed to generate signature key pair");
    signer
        .store(provider.storage())
        .expect("failed to store signature key pair");

    let credential_with_key = CredentialWithKey {
        credential: creator_credential.clone(),
        signature_key: signer.to_public_vec().into(),
    };

    MlsGroup::new(
        &provider,
        &signer,
        &MlsGroupCreateConfig::default(),
        credential_with_key,
    )
    .expect("failed to create MLS group")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_creation_and_member_add_poc() {
        let provider = OpenMlsRustCrypto::default();

        let alice_credential = create_credential(b"alice");
        let alice_signer =
            SignatureKeyPair::new(CIPHERSUITE.signature_algorithm()).expect("alice signer");
        alice_signer
            .store(provider.storage())
            .expect("store alice signer");

        let alice_credential_with_key = CredentialWithKey {
            credential: alice_credential.clone(),
            signature_key: alice_signer.to_public_vec().into(),
        };

        let mut group = MlsGroup::new(
            &provider,
            &alice_signer,
            &MlsGroupCreateConfig::default(),
            alice_credential_with_key,
        )
        .expect("create group");

        let bob_credential = create_credential(b"bob");
        let bob_signer =
            SignatureKeyPair::new(CIPHERSUITE.signature_algorithm()).expect("bob signer");
        bob_signer
            .store(provider.storage())
            .expect("store bob signer");

        let bob_credential_with_key = CredentialWithKey {
            credential: bob_credential,
            signature_key: bob_signer.to_public_vec().into(),
        };

        let bob_key_package = KeyPackage::builder()
            .key_package_extensions(Extensions::default())
            .build(CIPHERSUITE, &provider, &bob_signer, bob_credential_with_key)
            .expect("build bob key package");

        let (_commit, welcome, _group_info) = group
            .add_members(
                &provider,
                &alice_signer,
                core::slice::from_ref(bob_key_package.key_package()),
            )
            .expect("add bob to group");

        let _ = &welcome; // welcome (MlsMessageOut) is always present for new member

        group
            .merge_pending_commit(&provider)
            .expect("merge pending commit");

        assert_eq!(group.members().count(), 2);
    }
}
