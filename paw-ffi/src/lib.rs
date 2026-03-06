pub mod api;
pub use api::{AccountKeys, create_account, decrypt, encrypt};

mod frb_generated;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_account_key_sizes() {
        let keys = create_account();
        assert_eq!(keys.identity_key.len(), 32);
        assert_eq!(keys.identity_secret.len(), 32);
        assert_eq!(keys.signed_prekey.len(), 32);
        assert_eq!(keys.signed_prekey_secret.len(), 32);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let alice = create_account();
        let plaintext = b"hello paw e2ee".to_vec();

        let ciphertext = encrypt(alice.signed_prekey.clone(), plaintext.clone());
        assert!(
            ciphertext.len() > plaintext.len(),
            "ciphertext must be larger than plaintext"
        );
        assert_ne!(
            ciphertext[44..],
            plaintext[..],
            "ciphertext must differ from plaintext"
        );

        let decrypted = decrypt(alice.signed_prekey_secret, ciphertext);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_ciphertext_format() {
        let alice = create_account();
        let ciphertext = encrypt(alice.signed_prekey, b"test".to_vec());
        assert!(ciphertext.len() >= 12 + 32 + 20);
    }
}
