//! Comparison APK image encryption, not an uploader.
//! LX/LG2.AKp derives 80 bytes; LX/Kr5.A01 splits IV/cipher/MAC keys;
//! LX/Jq3 encrypts CBC/PKCS padding and appends the first ten MAC bytes.
use crate::{Error, Result};
use hkdf::hmac::{Hmac, Mac, digest::KeyInit};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

/// Prepared encrypted image. Persist the key privately before uploading.
pub struct EncryptedImage {
    /// Random per-file media key, distinct from account identity.
    pub media_key: Zeroizing<[u8; 32]>,
    /// CBC ciphertext followed by a ten-byte authentication tag.
    pub body: Vec<u8>,
    /// SHA-256 of the original file.
    pub plaintext_hash: [u8; 32],
    /// SHA-256 of the complete uploaded encrypted body.
    pub encrypted_hash: [u8; 32],
    /// Original byte length.
    pub plaintext_len: u64,
}
impl std::fmt::Debug for EncryptedImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EncryptedImage(<redacted>)")
    }
}

fn invalid() -> Error {
    Error::invalid("image", "invalid image input or encryption failure")
}

/// Authenticate an encrypted image without decrypting or displaying it.
pub fn verify_encrypted(
    body: &[u8],
    key: &[u8; 32],
    expected_hash: &[u8; 32],
    plaintext_len: u64,
) -> Result<()> {
    verify_kind(
        body,
        key,
        expected_hash,
        plaintext_len,
        b"WhatsApp Image Keys",
    )
}
fn verify_kind(
    body: &[u8],
    key: &[u8; 32],
    expected_hash: &[u8; 32],
    plaintext_len: u64,
    info: &[u8],
) -> Result<()> {
    if plaintext_len == 0
        || plaintext_len > 8 * 1024 * 1024
        || body.len() as u64 != (plaintext_len / 16 + 1) * 16 + 10
        || Sha256::digest(body).as_slice() != expected_hash
    {
        return Err(invalid());
    }
    let mut expanded = Zeroizing::new([0; 80]);
    hkdf::Hkdf::<Sha256>::new(None, key)
        .expand(info, &mut *expanded)
        .map_err(|_| invalid())?;
    let mut mac =
        <Hmac<Sha256> as KeyInit>::new_from_slice(&expanded[48..]).map_err(|_| invalid())?;
    mac.update(&expanded[..16]);
    mac.update(&body[..body.len() - 10]);
    mac.verify_truncated_left(&body[body.len() - 10..])
        .map_err(|_| invalid())
}

/// Authenticate and decrypt for a consuming viewer. Caller owns the durable
/// single-use transition; this primitive alone does not enforce view-once state.
pub fn decrypt(
    body: &[u8],
    key: &[u8; 32],
    encrypted_hash: &[u8; 32],
    plaintext_hash: &[u8; 32],
    plaintext_len: u64,
) -> Result<Zeroizing<Vec<u8>>> {
    decrypt_kind(
        body,
        key,
        encrypted_hash,
        plaintext_hash,
        plaintext_len,
        b"WhatsApp Image Keys",
    )
}

/// Authenticate and decrypt an audio attachment using its distinct key domain.
pub fn decrypt_audio(
    body: &[u8],
    key: &[u8; 32],
    encrypted_hash: &[u8; 32],
    plaintext_hash: &[u8; 32],
    plaintext_len: u64,
) -> Result<Zeroizing<Vec<u8>>> {
    decrypt_kind(
        body,
        key,
        encrypted_hash,
        plaintext_hash,
        plaintext_len,
        b"WhatsApp Audio Keys",
    )
}
/// Authenticate and decrypt video/video-note ciphertext. APK 2.26.37.71
/// `1qB.<clinit>` 0054..006a, 02e4..031a supplies this domain to both types.
pub fn decrypt_video(
    body: &[u8],
    key: &[u8; 32],
    encrypted_hash: &[u8; 32],
    plaintext_hash: &[u8; 32],
    plaintext_len: u64,
) -> Result<Zeroizing<Vec<u8>>> {
    decrypt_kind(
        body,
        key,
        encrypted_hash,
        plaintext_hash,
        plaintext_len,
        b"WhatsApp Video Keys",
    )
}
fn decrypt_kind(
    body: &[u8],
    key: &[u8; 32],
    encrypted_hash: &[u8; 32],
    plaintext_hash: &[u8; 32],
    plaintext_len: u64,
    info: &[u8],
) -> Result<Zeroizing<Vec<u8>>> {
    verify_kind(body, key, encrypted_hash, plaintext_len, info)?;
    let mut expanded = Zeroizing::new([0; 80]);
    hkdf::Hkdf::<Sha256>::new(None, key)
        .expand(info, &mut *expanded)
        .map_err(|_| invalid())?;
    let mut plaintext = Zeroizing::new(Vec::new());
    wacore_libsignal::crypto::aes_256_cbc_decrypt_into(
        &body[..body.len() - 10],
        &expanded[16..48],
        &expanded[..16],
        &mut plaintext,
    )
    .map_err(|_| invalid())?;
    if plaintext.len() as u64 != plaintext_len
        || Sha256::digest(&*plaintext).as_slice() != plaintext_hash
    {
        return Err(invalid());
    }
    Ok(plaintext)
}

/// Encrypt caller-provided image bytes. The 8 MiB ceiling is a local lab limit.
pub fn encrypt(bytes: &[u8]) -> Result<EncryptedImage> {
    encrypt_kind(bytes, b"WhatsApp Image Keys")
}

/// Comparison LX/1qB: voice notes use the audio key domain.
pub fn encrypt_audio(bytes: &[u8]) -> Result<EncryptedImage> {
    encrypt_kind(bytes, b"WhatsApp Audio Keys")
}

fn encrypt_kind(bytes: &[u8], info: &[u8]) -> Result<EncryptedImage> {
    if bytes.is_empty() || bytes.len() > 8 * 1024 * 1024 {
        return Err(invalid());
    }
    let mut key = Zeroizing::new([0u8; 32]);
    getrandom::fill(&mut *key).map_err(|_| invalid())?;
    encrypt_with_info(bytes, key, info)
}

#[cfg(test)]
fn encrypt_with_key(bytes: &[u8], key: Zeroizing<[u8; 32]>) -> Result<EncryptedImage> {
    encrypt_with_info(bytes, key, b"WhatsApp Image Keys")
}

fn encrypt_with_info(
    bytes: &[u8],
    key: Zeroizing<[u8; 32]>,
    info: &[u8],
) -> Result<EncryptedImage> {
    let mut expanded = Zeroizing::new([0u8; 80]);
    hkdf::Hkdf::<Sha256>::new(None, &*key)
        .expand(info, &mut *expanded)
        .map_err(|_| invalid())?;
    let mut body = Vec::new();
    wacore_libsignal::crypto::aes_256_cbc_encrypt_into(
        bytes,
        &expanded[16..48],
        &expanded[..16],
        &mut body,
    )
    .map_err(|_| invalid())?;
    let mut mac =
        <Hmac<Sha256> as KeyInit>::new_from_slice(&expanded[48..80]).map_err(|_| invalid())?;
    mac.update(&expanded[..16]);
    mac.update(&body);
    body.extend_from_slice(&mac.finalize().into_bytes()[..10]);
    Ok(EncryptedImage {
        media_key: key,
        plaintext_hash: Sha256::digest(bytes).into(),
        encrypted_hash: Sha256::digest(&body).into(),
        plaintext_len: bytes.len() as u64,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encrypted_download_checks_hash_mac_and_declared_length_without_decrypting() {
        let image = encrypt_with_key(b"private image", Zeroizing::new([7; 32])).unwrap();
        verify_encrypted(
            &image.body,
            &image.media_key,
            &image.encrypted_hash,
            image.plaintext_len,
        )
        .unwrap();
        assert!(
            verify_encrypted(
                &image.body,
                &[8; 32],
                &image.encrypted_hash,
                image.plaintext_len
            )
            .is_err()
        );
        let mut tampered = image.body.clone();
        tampered[0] ^= 1;
        let changed_hash = Sha256::digest(&tampered).into();
        assert!(
            verify_encrypted(
                &tampered,
                &image.media_key,
                &changed_hash,
                image.plaintext_len
            )
            .is_err()
        );
        assert!(
            verify_encrypted(&image.body, &image.media_key, &[0; 32], image.plaintext_len).is_err()
        );
        assert!(verify_encrypted(&image.body, &image.media_key, &image.encrypted_hash, 0).is_err());
    }
    #[test]
    fn audio_uses_separate_hkdf_domain() {
        let audio = encrypt_with_info(
            b"offline audio",
            Zeroizing::new([7; 32]),
            b"WhatsApp Audio Keys",
        )
        .unwrap();
        let image = encrypt_with_key(b"offline audio", Zeroizing::new([7; 32])).unwrap();
        assert_ne!(audio.body, image.body);
        let mut keys = [0; 80];
        hkdf::Hkdf::<Sha256>::new(None, &[7; 32])
            .expand(b"WhatsApp Audio Keys", &mut keys)
            .unwrap();
        let mut plain = Vec::new();
        wacore_libsignal::crypto::aes_256_cbc_decrypt_into(
            &audio.body[..audio.body.len() - 10],
            &keys[16..48],
            &keys[..16],
            &mut plain,
        )
        .unwrap();
        assert_eq!(plain, b"offline audio");
        assert_eq!(
            &*decrypt_audio(
                &audio.body,
                &[7; 32],
                &audio.encrypted_hash,
                &audio.plaintext_hash,
                audio.plaintext_len
            )
            .unwrap(),
            b"offline audio"
        );
        assert!(
            decrypt(
                &audio.body,
                &[7; 32],
                &audio.encrypted_hash,
                &audio.plaintext_hash,
                audio.plaintext_len
            )
            .is_err()
        );
        let mut tampered = audio.body.clone();
        tampered[0] ^= 1;
        assert!(
            decrypt_audio(
                &tampered,
                &[7; 32],
                &audio.encrypted_hash,
                &audio.plaintext_hash,
                audio.plaintext_len
            )
            .is_err()
        );
    }
    #[test]
    fn image_encryption_roundtrip_and_mac_cover_iv_and_ciphertext() {
        let image = encrypt_with_key(b"offline image bytes", Zeroizing::new([7; 32])).unwrap();
        let mut keys = [0; 80];
        hkdf::Hkdf::<Sha256>::new(None, &[7; 32])
            .expand(b"WhatsApp Image Keys", &mut keys)
            .unwrap();
        let split = image.body.len() - 10;
        let mut plain = Vec::new();
        wacore_libsignal::crypto::aes_256_cbc_decrypt_into(
            &image.body[..split],
            &keys[16..48],
            &keys[..16],
            &mut plain,
        )
        .unwrap();
        assert_eq!(plain, b"offline image bytes");
        let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(&keys[48..]).unwrap();
        mac.update(&keys[..16]);
        mac.update(&image.body[..split]);
        assert_eq!(&mac.finalize().into_bytes()[..10], &image.body[split..]);
        assert_eq!(image.body.len(), 42);
        assert!(!format!("{image:?}").contains("offline"));
        assert_ne!(
            encrypt(b"same").unwrap().body,
            encrypt(b"same").unwrap().body
        );
        assert!(encrypt(&[]).is_err());
        assert!(encrypt(&vec![0; 8 * 1024 * 1024 + 1]).is_err());
    }
}
