use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rand::RngCore;
use sha2::{Digest, Sha256};

fn derive_key(secret: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.finalize().into()
}

pub fn encrypt(secret: &str, plaintext: &str) -> Result<String> {
    let key = derive_key(secret);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| anyhow!(e.to_string()))?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow!(e.to_string()))?;
    let mut out = nonce_bytes.to_vec();
    out.extend(ct);
    Ok(STANDARD.encode(out))
}

pub fn decrypt(secret: &str, encoded: &str) -> Result<String> {
    let data = STANDARD.decode(encoded)?;
    if data.len() < 12 {
        return Err(anyhow!("invalid ciphertext"));
    }
    let (nonce_bytes, ct) = data.split_at(12);
    let key = derive_key(secret);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| anyhow!(e.to_string()))?;
    let pt = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ct)
        .map_err(|e| anyhow!(e.to_string()))?;
    Ok(String::from_utf8(pt)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encrypt_decrypt_roundtrip() {
        let secret = "test-secret-key-0123456789abcdef";
        let pt = "refresh-token-value";
        let enc = encrypt(secret, pt).unwrap();
        assert_ne!(enc, pt);
        let dec = decrypt(secret, &enc).unwrap();
        assert_eq!(dec, pt);
    }
    #[test]
    fn different_ciphertexts_for_same_input() {
        let secret = "test-secret-key-0123456789abcdef";
        let a = encrypt(secret, "same").unwrap();
        let b = encrypt(secret, "same").unwrap();
        assert_ne!(a, b);
    }
}
