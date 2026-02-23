use std::ptr::NonNull;

use crate::error::{BiometricError, Result};

/// Wrapper around Linux kernel BCH library (bchlib-sys).
///
/// BCH field parameters:
///   m=11  =>  GF(2^11)  =>  max codeword length n = 2^11 - 1 = 2047 bits
///
/// For BCH(2047, 512, 180):
///   ECC bits  = m * t = 11 * 180 = 1980 bits
///   ECC bytes = ceil(1980 / 8) = 248 bytes
///   Constraint check: t * m <= n  =>  1980 <= 2047  (satisfied)
///
/// Why m=10 was invalid:
///   m=10 gives n_max=1023, and 180 * 10 = 1800 > 1023 (violated the BCH bound,
///   causing bchlib_sys::init_bch to return null).
#[derive(Debug)]
pub struct BchEngine {
    ctrl: NonNull<bchlib_sys::bch_control>,
    m: i32,
    t: i32,
}

impl BchEngine {
    /// Initialize BCH engine with Galois field order (m) and error correction capability (t).
    ///
    /// For production use call with m=11, t=180 (BCH(2047, 512, 180)).
    pub fn new(m: i32, t: i32) -> Result<Self> {
        if m <= 0 || t <= 0 {
            return Err(BiometricError::InvalidBchParams(format!(
                "m and t must be positive integers, got m={}, t={}",
                m, t
            )));
        }

        // n_max for this field order
        let max_n = (1i64 << m) - 1;

        // Ensure field supports the required codeword length of 2047 bits
        if max_n < 2047 {
            return Err(BiometricError::InvalidBchParams(format!(
                "m={} only supports codewords up to {} bits, need 2047",
                m, max_n
            )));
        }

        // Validate BCH bound: t * m must not exceed n_max
        let t_m_product = t as i64 * m as i64;
        if t_m_product > max_n {
            return Err(BiometricError::InvalidBchParams(format!(
                "BCH bound violated: t * m = {} * {} = {} exceeds n_max={}. \
                 Increase m or decrease t.",
                t, m, t_m_product, max_n
            )));
        }

        let ptr = unsafe { bchlib_sys::init_bch(m, t, 0) };
        let ctrl = NonNull::new(ptr).ok_or_else(|| {
            BiometricError::InvalidBchParams(format!(
                "BCH initialization failed for m={}, t={}. \
                 The underlying C library rejected these parameters.",
                m, t
            ))
        })?;

        Ok(Self { ctrl, m, t })
    }

    /// Calculate number of ECC bytes required.
    ///
    /// ECC bits  = m * t
    /// ECC bytes = ceil(m * t / 8)
    ///
    /// For m=11, t=180: ceil(1980 / 8) = 248 bytes.
    pub fn ecc_bytes(&self) -> usize {
        (self.m as usize * self.t as usize).div_ceil(8)
    }

    /// Encode message bytes and return ECC parity bytes.
    pub fn encode(&self, msg: &[u8]) -> Result<Vec<u8>> {
        let ecc_len = self.ecc_bytes();
        let mut ecc = vec![0u8; ecc_len];

        unsafe {
            bchlib_sys::encode_bch(
                self.ctrl.as_ptr(),
                msg.as_ptr(),
                msg.len() as u32,
                ecc.as_mut_ptr(),
            );
        }

        Ok(ecc)
    }

    /// Decode received message and ECC, applying in-place bit error corrections.
    ///
    /// Returns the number of bit errors corrected.
    /// Returns an error if the number of errors exceeds t (uncorrectable).
    pub fn decode_and_correct(&self, msg: &mut [u8], recv_ecc: &[u8]) -> Result<usize> {
        let mut errloc = vec![0u32; self.t as usize];

        let nerr = unsafe {
            bchlib_sys::decode_bch(
                self.ctrl.as_ptr(),
                msg.as_ptr(),
                msg.len() as u32,
                recv_ecc.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                errloc.as_mut_ptr(),
            )
        };

        // nerr < 0 means the errors are uncorrectable (exceed t)
        if nerr < 0 {
            return Err(BiometricError::EccDecode(format!(
                "Uncorrectable errors detected. More than {} bit errors are present. \
                 Biometric does not match enrolled template.",
                self.t
            )));
        }

        let corrected = nerr as usize;

        if corrected == 0 {
            return Ok(0);
        }

        // Apply each correction by flipping the identified bit
        let total_msg_bits = msg.len() * 8;
        for &pos in errloc.iter().take(corrected) {
            let bit_index = pos as usize;
            if bit_index < total_msg_bits {
                let byte_index = bit_index / 8;
                let bit_in_byte = bit_index % 8;
                msg[byte_index] ^= 1u8 << bit_in_byte;
            }
        }

        Ok(corrected)
    }
}

impl Drop for BchEngine {
    fn drop(&mut self) {
        unsafe {
            bchlib_sys::free_bch(self.ctrl.as_ptr());
        }
    }
}

unsafe impl Send for BchEngine {}
unsafe impl Sync for BchEngine {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bch_engine_initialization() {
        let engine = BchEngine::new(11, 180);
        assert!(engine.is_ok());
    }

    #[test]
    fn bch_engine_ecc_bytes_calculation() {
        let engine = BchEngine::new(11, 180).unwrap();
        // m=11, t=180 => 11 * 180 = 1980 bits => ceil(1980 / 8) = 248 bytes
        assert_eq!(engine.ecc_bytes(), 248);
    }

    #[test]
    fn bch_engine_rejects_invalid_m() {
        // m=10 violates the BCH bound for t=180: 180 * 10 = 1800 > 1023
        let engine = BchEngine::new(10, 180);
        assert!(engine.is_err());
    }

    #[test]
    fn bch_engine_rejects_nonpositive_params() {
        assert!(BchEngine::new(0, 180).is_err());
        assert!(BchEngine::new(11, 0).is_err());
    }
}
