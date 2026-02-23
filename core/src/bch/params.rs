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

    pub fn new_1023_512(t: usize) -> Self {
        Self { n: 1023, k: 512, t }
    }

    pub fn validate(&self) -> Result<()> {
        if self.n != 1023 || self.k != 512 {
            return Err(BiometricError::InvalidBchParams(format!(
                "this profile supports only n=1023, k=512; got n={}, k={}",
                self.n, self.k
            )));
        }

        // Bound is enforced to current backend profile guarantees.
        if self.t == 0 || self.t > 15 {
            return Err(BiometricError::InvalidBchParams(format!(
                "true BCH correction in this profile supports t in 1..=15; got {}",
                self.t
            )));
        }

        Ok(())
    }
}

impl Default for BchParams {
    fn default() -> Self {
        Self::new_1023_512(15)
    }
}
