use std::ptr::NonNull;

use crate::error::{BiometricError, Result};

#[derive(Debug)]
pub struct BchEngine {
    ctrl: NonNull<bchlib_sys::bch_control>,
    m: i32,
    t: i32,
}

impl BchEngine {
    pub fn new(m: i32, t: i32) -> Result<Self> {
        if m <= 0 || t <= 0 {
            return Err(BiometricError::InvalidBchParams(format!(
                "m and t must be > 0, got m={}, t={}",
                m, t
            )));
        }

        let ptr = unsafe { bchlib_sys::init_bch(m, t, 0) };
        let ctrl = NonNull::new(ptr).ok_or_else(|| {
            BiometricError::InvalidBchParams(format!("init_bch failed for m={}, t={}", m, t))
        })?;

        Ok(Self { ctrl, m, t })
    }

    pub fn ecc_bytes(&self) -> usize {
        // Linux BCH parity bits ~= m*t; implementation stores packed bytes.
        ((self.m as usize) * (self.t as usize)).div_ceil(8)
    }

    pub fn encode(&self, msg: &[u8]) -> Result<Vec<u8>> {
        let mut ecc = vec![0u8; self.ecc_bytes()];
        unsafe {
            bchlib_sys::encode_bch(self.ctrl.as_ptr(), msg.as_ptr(), msg.len() as u32, ecc.as_mut_ptr());
        }
        Ok(ecc)
    }

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

        if nerr < 0 {
            return Err(BiometricError::EccDecode(format!(
                "decode_bch failed with status {}",
                nerr
            )));
        }

        let corrected = nerr as usize;
        if corrected == 0 {
            return Ok(0);
        }

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
