use hkdf::Hkdf;
use sha2::Sha256;

use crate::error::{BiometricError, Result};

pub const KEY_SIZE: usize = 32;

pub fn derive_key_256(
    input_key_material: &[u8],
    salt: &[u8],
    info: &[u8],
) -> Result<[u8; KEY_SIZE]> {
    let hk = Hkdf::<Sha256>::new(Some(salt), input_key_material);
    let mut okm = [0u8; KEY_SIZE];
    hk.expand(info, &mut okm)
        .map_err(|e| BiometricError::Validation(format!("hkdf expand failed: {e}")))?;
    Ok(okm)
}
