use crate::error::{BiometricError, Result};

/// BCH code parameters for the biometric key derivation pipeline.
///
/// Production configuration: BCH(2047, 512, 180)
///
/// Why these parameters:
///   - k=512 matches AdaFace 512-dimensional embeddings (1 bit per dimension,
///     sign-based quantization: value >= 0.0 => 1, value < 0.0 => 0)
///   - n=2047 is the codeword length for GF(2^11): n = 2^11 - 1 = 2047
///   - t=180 corrects up to 180 bit errors (~8.8% of 2047 bits)
///
/// Error rate targets (derived from mean-of-10-frames averaging strategy):
///   Same person  (enrollment vs recovery): 3-8%  (~15-40 bit errors)  CORRECTABLE
///   Different people:                      48-52% (~982-1064 bit errors) REJECTED
///
/// BCH bound verification:
///   t * m = 180 * 11 = 1980 <= 2047 = n  (satisfied)
///
/// ECC overhead:
///   ECC bits  = m * t = 11 * 180 = 1980 bits
///   ECC bytes = ceil(1980 / 8)   = 248 bytes
///   Parity bits stored in codeword: n - k = 2047 - 512 = 1535 bits
#[derive(Debug, Clone, Copy)]
pub struct BchParams {
    pub n: usize,
    pub k: usize,
    pub t: usize,
}

impl BchParams {
    /// Production parameters: BCH(2047, 512, t).
    ///
    /// Recommended t values:
    ///   150 - conservative, 8.2% tolerance  (stricter security)
    ///   180 - balanced,     8.8% tolerance  (production default)
    ///   186 - maximum valid for m=11: floor(2047 / 11) = 186
    ///
    /// Note: t=186 is the hard ceiling imposed by the BCH bound (t * m <= n).
    /// Attempting t > 186 with m=11 will cause BchEngine::new to return an error.
    pub fn new_2047_512(t: usize) -> Self {
        Self { n: 2047, k: 512, t }
    }

    /// Legacy parameters: BCH(255, 128, t).
    ///
    /// Retained for compatibility with older enrolled data only.
    /// Do not use for new enrollments.
    pub fn new_255_128(t: usize) -> Self {
        Self { n: 255, k: 128, t }
    }

    pub fn validate(&self) -> Result<()> {
        // Only the 2047/512 production profile is supported for new work
        if self.n != 2047 || self.k != 512 {
            return Err(BiometricError::InvalidBchParams(format!(
                "Production profile requires n=2047, k=512; got n={}, k={}",
                self.n, self.k
            )));
        }

        if self.t == 0 {
            return Err(BiometricError::InvalidBchParams(
                "t must be greater than 0".to_string(),
            ));
        }

        // Hard ceiling from BCH bound with m=11: t * 11 <= 2047 => t <= 186
        let max_t = self.n / 11; // floor(2047 / 11) = 186
        if self.t > max_t {
            return Err(BiometricError::InvalidBchParams(format!(
                "t={} exceeds BCH bound maximum {} for BCH({}, {}) with m=11. \
                 t * m must be <= n: {} * 11 = {} > {}",
                self.t,
                max_t,
                self.n,
                self.k,
                self.t,
                self.t * 11,
                self.n
            )));
        }

        // Minimum t for reliable biometric recovery given expected 3-8% error rate.
        // With n=2047 and expected ~40 bit errors worst case, t=150 provides
        // 150 error budget which exceeds the 40-bit worst case with margin.
        if self.t < 150 {
            return Err(BiometricError::InsecureConfiguration(format!(
                "t={} is too low for reliable biometric key recovery. \
                 Minimum required: 150 (covers up to 7.3% of 2047 bits). \
                 Expected same-person error rate after mean averaging: 3-8%.",
                self.t
            )));
        }

        Ok(())
    }
}

impl Default for BchParams {
    /// Default production parameters: BCH(2047, 512, 180).
    ///
    /// Tolerates up to 180 bit errors (~8.8% of 2047 bits).
    /// Safety margin over expected 3-8% same-person error rate is sufficient
    /// for production use with the 10-frame mean embedding strategy.
    fn default() -> Self {
        Self::new_2047_512(180)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_params_are_valid() {
        let params = BchParams::default();
        assert!(params.validate().is_ok());
        assert_eq!(params.n, 2047);
        assert_eq!(params.k, 512);
        assert_eq!(params.t, 180);
    }

    #[test]
    fn conservative_params_are_valid() {
        let params = BchParams::new_2047_512(150);
        assert!(params.validate().is_ok());
    }

    #[test]
    fn aggressive_params_are_valid() {
        // t=186 is the maximum permitted by the BCH bound with m=11
        let params = BchParams::new_2047_512(186);
        assert!(params.validate().is_ok());
    }

    #[test]
    fn too_low_t_is_rejected() {
        let params = BchParams::new_2047_512(100);
        assert!(params.validate().is_err());
    }

    #[test]
    fn too_high_t_is_rejected() {
        // t=187 exceeds floor(2047 / 11) = 186
        let params = BchParams::new_2047_512(187);
        assert!(params.validate().is_err());
    }

    #[test]
    fn wrong_n_is_rejected() {
        let params = BchParams {
            n: 1023,
            k: 512,
            t: 180,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn wrong_k_is_rejected() {
        let params = BchParams {
            n: 2047,
            k: 256,
            t: 180,
        };
        assert!(params.validate().is_err());
    }
}
