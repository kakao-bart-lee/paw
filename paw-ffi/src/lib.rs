use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

pub struct AccountKeys {
    pub identity_key: Vec<u8>,
    pub identity_secret: Vec<u8>,
    pub signed_prekey: Vec<u8>,
    pub signed_prekey_secret: Vec<u8>,
}

pub fn create_account() -> AccountKeys {
    let mut rng = rand::rngs::OsRng;

    let identity_secret = StaticSecret::random_from_rng(&mut rng);
    let identity_key = PublicKey::from(&identity_secret);

    let signed_prekey_secret = StaticSecret::random_from_rng(&mut rng);
    let signed_prekey = PublicKey::from(&signed_prekey_secret);

    AccountKeys {
        identity_key: identity_key.as_bytes().to_vec(),
        identity_secret: identity_secret.to_bytes().to_vec(),
        signed_prekey: signed_prekey.as_bytes().to_vec(),
        signed_prekey_secret: signed_prekey_secret.to_bytes().to_vec(),
    }
}

fn derive_aes_key(shared_secret: &[u8; 32]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(None, shared_secret);
    let mut key = [0u8; 32];
    hk.expand(b"paw-ffi-aes-256-gcm-key", &mut key)
        .expect("HKDF expand failed");
    key
}

pub fn encrypt(their_signed_prekey: Vec<u8>, plaintext: Vec<u8>) -> Vec<u8> {
    let their_pk: [u8; 32] = their_signed_prekey
        .as_slice()
        .try_into()
        .expect("their_signed_prekey must be 32 bytes");
    let their_pk = PublicKey::from(their_pk);

    let ephemeral_secret = EphemeralSecret::random_from_rng(rand::rngs::OsRng);
    let ephemeral_pubkey = PublicKey::from(&ephemeral_secret);

    let shared_secret = ephemeral_secret.diffie_hellman(&their_pk);
    let key_bytes = derive_aes_key(shared_secret.as_bytes());

    let cipher = Aes256Gcm::new_from_slice(&key_bytes).expect("invalid AES key length");
    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let encrypted = cipher
        .encrypt(nonce, plaintext.as_ref())
        .expect("AES-GCM encrypt failed");

    let mut out = Vec::with_capacity(12 + 32 + encrypted.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(ephemeral_pubkey.as_bytes());
    out.extend_from_slice(&encrypted);
    out
}

pub fn decrypt(my_signed_prekey_secret: Vec<u8>, ciphertext: Vec<u8>) -> Vec<u8> {
    assert!(ciphertext.len() >= 44, "ciphertext too short");

    let nonce_bytes = &ciphertext[0..12];
    let ephemeral_pubkey_bytes: [u8; 32] = ciphertext[12..44]
        .try_into()
        .expect("ephemeral pubkey must be 32 bytes");
    let encrypted = &ciphertext[44..];

    let my_secret_bytes: [u8; 32] = my_signed_prekey_secret
        .as_slice()
        .try_into()
        .expect("my_signed_prekey_secret must be 32 bytes");
    let my_secret = StaticSecret::from(my_secret_bytes);
    let ephemeral_pubkey = PublicKey::from(ephemeral_pubkey_bytes);

    let shared_secret = my_secret.diffie_hellman(&ephemeral_pubkey);
    let key_bytes = derive_aes_key(shared_secret.as_bytes());

    let cipher = Aes256Gcm::new_from_slice(&key_bytes).expect("invalid AES key length");
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, encrypted)
        .expect("AES-GCM decrypt failed")
}

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
