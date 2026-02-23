use std::ptr::NonNull;

use crate::error::{BiometricError, Result};

/// Wrapper around Linux kernel BCH library (bchlib-sys)
/// 
/// CRITICAL LIMITATION: The bchlib backend uses m=10 (Galois field GF(2^10))
/// which limits us to codewords of length n=2^10-1=1023 bits.
/// 
/// For our BCH(1023, 512, t) system, the backend must support t up to ~200.
/// The actual m*t product determines ECC byte length.
#[derive(Debug)]
pub struct BchEngine {
    ctrl: NonNull<bchlib_sys::bch_control>,
    m: i32,
    t: i32,
}

impl BchEngine {
    /// Initialize BCH engine with given Galois field order (m) and error correction capability (t)
    /// 
    /// For BCH(1023, 512, 180):
    /// - m = 10 (field GF(2^10), supports n up to 1023)
    /// - t = 180 (can correct up to 180 bit errors)
    pub fn new(m: i32, t: i32) -> Result<Self> {
        if m <= 0 || t <= 0 {
            return Err(BiometricError::InvalidBchParams(format!(
                "m and t must be positive integers, got m={}, t={}",
                m, t
            )));
        }

        // Validate that m supports our codeword length
        // For m=10, max n = 2^10 - 1 = 1023
        let max_n = (1 << m) - 1;
        if max_n < 1023 {
            return Err(BiometricError::InvalidBchParams(format!(
                "m={} only supports codewords up to {} bits, need 1023",
                m, max_n
            )));
        }

        let ptr = unsafe { bchlib_sys::init_bch(m, t, 0) };
        let ctrl = NonNull::new(ptr).ok_or_else(|| {
            BiometricError::InvalidBchParams(format!(
                "BCH initialization failed for m={}, t={}. Backend may not support these parameters.",
                m, t
            ))
        })?;

        Ok(Self { ctrl, m, t })
    }

    /// Calculate number of ECC bytes required
    /// ECC bits = m * t (approximately)
    /// ECC bytes = ceil((m * t) / 8)
    pub fn ecc_bytes(&self) -> usize {
        ((self.m as usize) * (self.t as usize)).div_ceil(8)
    }

    /// Encode message to generate ECC parity bytes
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

    /// Decode and correct errors in message using received ECC
    /// 
    /// Returns: Number of bit errors corrected
    /// Error: If errors exceed correction capability (>t)
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

        // nerr < 0 means uncorrectable errors
        if nerr < 0 {
            return Err(BiometricError::EccDecode(format!(
                "Uncorrectable errors detected. More than {} bit errors present.",
                self.t
            )));
        }

        let corrected = nerr as usize;
        
        // If no errors, return early
        if corrected == 0 {
            return Ok(0);
        }

        // Apply corrections
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
        let engine = BchEngine::new(10, 180);
        assert!(engine.is_ok());
    }

    #[test]
    fn bch_engine_ecc_bytes_calculation() {
        let engine = BchEngine::new(10, 180).unwrap();
        // m=10, t=180 => 10*180 = 1800 bits = 225 bytes
        assert_eq!(engine.ecc_bytes(), 225);
    }
}
