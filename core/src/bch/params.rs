use crate::error::{BiometricError, Result};

#[derive(Debug, Clone, Copy)]
pub struct BchParams {
    pub n: usize,
    pub k: usize,
    pub t: usize,
}

impl BchParams {
    /// BCH(1023, 512, 180) - Production parameters for 512-bit AdaFace embeddings
    /// Corrects up to 180 bit errors (~17.5% of 1023 bits)
    /// Expected same-person errors: 3-8% (15-40 bits)
    /// Expected different-people errors: 48-52% (245-266 bits)
    pub fn new_1023_512(t: usize) -> Self {
        Self { n: 1023, k: 512, t }
    }

    /// Legacy 255-128 system (not recommended for production)
    pub fn new_255_128(t: usize) -> Self {
        Self { n: 255, k: 128, t }
    }

    pub fn validate(&self) -> Result<()> {
        // Validate n and k match expected profile
        if self.n != 1023 || self.k != 512 {
            return Err(BiometricError::InvalidBchParams(format!(
                "Current profile supports only n=1023, k=512; got n={}, k={}",
                self.n, self.k
            )));
        }

        // Validate error correction capability
        // For BCH, theoretical max t is approximately (n-k)/2
        // For (1023, 512), max t is ~255
        // We use t=180 for production (17.5% tolerance)
        if self.t == 0 {
            return Err(BiometricError::InvalidBchParams(
                "t must be greater than 0".to_string()
            ));
        }

        let max_t = (self.n - self.k) / 2;
        if self.t > max_t {
            return Err(BiometricError::InvalidBchParams(format!(
                "t={} exceeds theoretical maximum {} for BCH({}, {})",
                self.t, max_t, self.n, self.k
            )));
        }

        // Production constraint: t should be appropriate for biometric variance
        // Minimum: t=150 (conservative, 14.6% tolerance)
        // Recommended: t=180 (balanced, 17.5% tolerance)
        // Maximum: t=200 (aggressive, 19.5% tolerance)
        if self.t < 150 {
            return Err(BiometricError::InsecureConfiguration(format!(
                "t={} is too low for reliable biometric recovery. Minimum recommended: 150",
                self.t
            )));
        }

        if self.t > 200 {
            return Err(BiometricError::InsecureConfiguration(format!(
                "t={} is too high, may accept different people. Maximum recommended: 200",
                self.t
            )));
        }

        Ok(())
    }
}

impl Default for BchParams {
    /// Default production parameters: BCH(1023, 512, 180)
    /// Tolerates up to 17.5% bit errors
    fn default() -> Self {
        Self::new_1023_512(180)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_params_are_valid() {
        let params = BchParams::default();
        assert!(params.validate().is_ok());
        assert_eq!(params.n, 1023);
        assert_eq!(params.k, 512);
        assert_eq!(params.t, 180);
    }

    #[test]
    fn conservative_params_are_valid() {
        let params = BchParams::new_1023_512(150);
        assert!(params.validate().is_ok());
    }

    #[test]
    fn aggressive_params_are_valid() {
        let params = BchParams::new_1023_512(200);
        assert!(params.validate().is_ok());
    }

    #[test]
    fn too_low_t_is_rejected() {
        let params = BchParams::new_1023_512(100);
        assert!(params.validate().is_err());
    }

    #[test]
    fn too_high_t_is_rejected() {
        let params = BchParams::new_1023_512(250);
        assert!(params.validate().is_err());
    }
}
