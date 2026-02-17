use crate::error::{BiometricError, Result};

#[derive(Debug, Clone, Copy)]
pub struct BchParams {
    pub n: usize,
    pub k: usize,
    pub t: usize,
}

impl BchParams {
    pub fn new_255_128(t: usize) -> Self {
        Self { n: 255, k: 128, t }
    }

    pub fn validate(&self) -> Result<()> {
        if self.n != 255 || self.k != 128 {
            return Err(BiometricError::InvalidBchParams(format!(
                "this backend currently supports only n=255, k=128; got n={}, k={}",
                self.n, self.k
            )));
        }

        if self.t == 0 || self.t > 127 {
            return Err(BiometricError::InvalidBchParams(format!(
                "t must be in 1..=127 for n=255, got {}",
                self.t
            )));
        }

        Ok(())
    }
}

impl Default for BchParams {
    fn default() -> Self {
        Self::new_255_128(90)
    }
}
