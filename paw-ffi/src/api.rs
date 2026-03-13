#![allow(unexpected_cfgs)]

#[derive(Clone, Debug)]
pub struct AccountKeys {
    pub identity_key: Vec<u8>,
    pub identity_secret: Vec<u8>,
    pub signed_prekey: Vec<u8>,
    pub signed_prekey_secret: Vec<u8>,
}

#[flutter_rust_bridge::frb(sync)]
pub fn create_account() -> AccountKeys {
    let keys = paw_core::create_account();

    AccountKeys {
        identity_key: keys.identity_key,
        identity_secret: keys.identity_secret,
        signed_prekey: keys.signed_prekey,
        signed_prekey_secret: keys.signed_prekey_secret,
    }
}

#[flutter_rust_bridge::frb(sync)]
pub fn encrypt(their_signed_prekey: Vec<u8>, plaintext: Vec<u8>) -> Vec<u8> {
    paw_core::encrypt(their_signed_prekey, plaintext)
}

#[flutter_rust_bridge::frb(sync)]
pub fn decrypt(my_signed_prekey_secret: Vec<u8>, ciphertext: Vec<u8>) -> Vec<u8> {
    paw_core::decrypt(my_signed_prekey_secret, ciphertext)
}
