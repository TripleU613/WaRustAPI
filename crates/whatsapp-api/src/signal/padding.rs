//! Application framing after authenticated Signal/CBC decryption, not PKCS padding.
use crate::{Error, Result};
use zeroize::Zeroizing;

pub(super) fn unpad(plaintext: Vec<u8>) -> Result<Zeroizing<Vec<u8>>> {
    let mut plaintext = Zeroizing::new(plaintext);
    // APK43.72 EHq.A00: unsigned last byte, >0 and strictly less than
    // plaintext length, then prefix copy. No repeated-byte check or16 cap.
    // Empty output is status-1000; zero/entire/oversized padding is-10000.
    // Neither is eligible in the retained EtY crypto retry predicate.
    let count = plaintext.last().copied().unwrap_or(0) as usize;
    if count == 0 || count >= plaintext.len() {
        return Err(Error::SignalInvalidApplicationPadding);
    }
    let length = plaintext.len() - count;
    plaintext.truncate(length);
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_count_admission_accepts_nonrepeating_and_full_byte_range() {
        assert_eq!(&*unpad(vec![42, 99, 1, 2]).unwrap(), &[42, 99]);
        for count in [1u8, 16, 17, 20, 255] {
            let mut bytes = vec![42];
            bytes.resize(1 + usize::from(count), 99);
            *bytes.last_mut().unwrap() = count;
            assert_eq!(&*unpad(bytes).unwrap(), &[42]);
        }
        for bytes in [
            vec![],
            vec![0],
            vec![42, 0],
            vec![1],
            vec![2, 2],
            vec![42, 3],
            vec![255],
        ] {
            assert!(matches!(
                unpad(bytes),
                Err(Error::SignalInvalidApplicationPadding)
            ));
        }
    }
}
