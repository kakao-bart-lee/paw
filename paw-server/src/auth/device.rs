use base64::{engine::general_purpose::STANDARD, Engine as _};

pub fn decode_ed25519_public_key(public_key_base64: &str) -> Result<Vec<u8>, String> {
    let key = STANDARD
        .decode(public_key_base64)
        .map_err(|_| "invalid base64 encoding".to_string())?;

    if key.len() != 32 {
        return Err("ed25519 public key must be 32 bytes".to_string());
    }

    Ok(key)
}
