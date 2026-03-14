use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

const NONCE_LEN: usize = 12;
const EPHEMERAL_PUBKEY_LEN: usize = 32;
const CIPHERTEXT_OVERHEAD: usize = NONCE_LEN + EPHEMERAL_PUBKEY_LEN;
const HKDF_INFO: &[u8] = b"paw-ffi-aes-256-gcm-key";

#[derive(Clone, Debug)]
pub struct AccountKeys {
    pub identity_key: Vec<u8>,
    pub identity_secret: Vec<u8>,
    pub signed_prekey: Vec<u8>,
    pub signed_prekey_secret: Vec<u8>,
}

pub fn create_account() -> AccountKeys {
    let identity_secret = StaticSecret::random_from_rng(rand::rngs::OsRng);
    let identity_key = PublicKey::from(&identity_secret);

    let signed_prekey_secret = StaticSecret::random_from_rng(rand::rngs::OsRng);
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
    hk.expand(HKDF_INFO, &mut key).expect("HKDF expand failed");
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
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let encrypted = cipher
        .encrypt(nonce, plaintext.as_ref())
        .expect("AES-GCM encrypt failed");

    let mut out = Vec::with_capacity(CIPHERTEXT_OVERHEAD + encrypted.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(ephemeral_pubkey.as_bytes());
    out.extend_from_slice(&encrypted);
    out
}

pub fn decrypt(my_signed_prekey_secret: Vec<u8>, ciphertext: Vec<u8>) -> Vec<u8> {
    assert!(
        ciphertext.len() >= CIPHERTEXT_OVERHEAD,
        "ciphertext too short"
    );

    let nonce_bytes = &ciphertext[..NONCE_LEN];
    let ephemeral_pubkey_bytes: [u8; 32] = ciphertext[NONCE_LEN..CIPHERTEXT_OVERHEAD]
        .try_into()
        .expect("ephemeral pubkey must be 32 bytes");
    let encrypted = &ciphertext[CIPHERTEXT_OVERHEAD..];

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
    use super::{create_account, decrypt, encrypt, CIPHERTEXT_OVERHEAD};

    #[test]
    fn create_account_returns_expected_key_sizes() {
        let keys = create_account();

        assert_eq!(keys.identity_key.len(), 32);
        assert_eq!(keys.identity_secret.len(), 32);
        assert_eq!(keys.signed_prekey.len(), 32);
        assert_eq!(keys.signed_prekey_secret.len(), 32);
    }

    #[test]
    fn encrypt_then_decrypt_round_trips() {
        let recipient = create_account();
        let plaintext = b"hello from paw-core".to_vec();

        let ciphertext = encrypt(recipient.signed_prekey.clone(), plaintext.clone());
        let decrypted = decrypt(recipient.signed_prekey_secret, ciphertext);

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn ciphertext_format_matches_current_ffi_contract() {
        let recipient = create_account();
        let ciphertext = encrypt(recipient.signed_prekey, b"test".to_vec());

        assert!(ciphertext.len() >= CIPHERTEXT_OVERHEAD + 20);
    }
}
