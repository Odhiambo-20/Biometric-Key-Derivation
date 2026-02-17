#![deny(missing_debug_implementations)]

pub mod bch;
pub mod error;
pub mod ffi;
pub mod fuzzy_extractor;
pub mod hash;
pub mod quantization;
pub mod utils;

pub use bch::params::BchParams;
pub use error::{BiometricError, Result};
pub use fuzzy_extractor::{enroll, recover, EnrollmentOutput, HelperData};
pub use quantization::QuantizationMethod;
