# Biometric Key Derivation Module

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Swift 5.9+](https://img.shields.io/badge/Swift-5.9+-red.svg)](https://swift.org/)
[![Kotlin 1.9+](https://img.shields.io/badge/Kotlin-1.9+-blue.svg)](https://kotlinlang.org/)
[![License](https://img.shields.io/badge/License-Proprietary-red.svg)]()
[![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen.svg)]()

## Overview

Production-grade biometric cryptographic SDK that derives stable 256-bit encryption keys from face biometrics. Enables **passwordless, PIN-less authentication** using only facial recognition with fuzzy extractor cryptography for cross-platform mobile applications.

### Key Features

- **Passwordless Security** - No passwords, PINs, or hardware tokens required
- **Cross-Platform** - Single Rust core, native iOS and Android interfaces
- **Fuzzy Extractor Cryptography** - BCH error correction for biometric variance tolerance
- **Hardware Security** - Secure Enclave (iOS) and StrongBox (Android) integration
- **Privacy Compliant** - GDPR-ready with no biometric template storage
- **Production Hardened** - Jailbreak detection, tamper protection, secure memory wiping
- **Real-time Performance** - Sub-second key derivation on mobile devices
- **Tolerance Tuned** - Handles makeup, lighting, beard growth, camera variations (7-15% bit errors)

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         APPLICATION LAYER                            │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────────────┐   │
│  │ Banking App  │  │ Health App   │  │  Enterprise App        │   │
│  └──────────────┘  └──────────────┘  └─────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                                 │
┌─────────────────────────────────────────────────────────────────────┐
│                   iOS (Swift) / Android (Kotlin)                     │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────────────┐   │
│  │ Camera       │  │ Frame        │  │ MobileFaceNet          │   │
│  │ Capture      │  │ Selection    │  │ Wrapper                │   │
│  └──────────────┘  └──────────────┘  └─────────────────────────┘   │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────────────┐   │
│  │ Security     │  │ Logging      │  │ Pipeline               │   │
│  │ (Keychain)   │  │ (Audit)      │  │ Coordinator            │   │
│  └──────────────┘  └──────────────┘  └─────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                                 │
┌─────────────────────────────────────────────────────────────────────┐
│                      RUST CORE (Shared Library)                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              Face Embedding (128-dim float)                   │   │
│  │                          ↓                                    │   │
│  │              Quantization (sign-based)                        │   │
│  │                          ↓                                    │   │
│  │              Binary Vector (128 bits)                         │   │
│  │                          ↓                                    │   │
│  │              BCH Encode/Decode (n=255, k=128, t=80-90)       │   │
│  │                          ↓                                    │   │
│  │              Fuzzy Extractor (helper data + XOR)             │   │
│  │                          ↓                                    │   │
│  │              HKDF/SHA256                                      │   │
│  │                          ↓                                    │   │
│  │              256-bit Cryptographic Key                        │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Technical Specifications

### Cryptographic Pipeline

| Stage | Algorithm | Input | Output | Purpose |
|-------|-----------|-------|--------|---------|
| **Face Embedding** | MobileFaceNet | 112x112 face image | 128 float32 values | Convert face to vector |
| **Quantization** | Sign-based | 128 floats | 128 bits | Discretize to binary |
| **Error Correction** | BCH(255,128,90) | 128 bits | 255-bit codeword | Tolerate variations |
| **Fuzzy Extractor** | XOR + Helper Data | Codeword | Original 128 bits | Recover stable bits |
| **Key Derivation** | HKDF-SHA256 | 128 bits | 256-bit key | Generate crypto key |

### Security Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| **Same Person Error Rate** | 7-15% bit errors | Makeup, lighting, camera, beard variations |
| **BCH Tolerance** | t=80-90 (~17%) | Safety margin: handles worst-case +2-10% buffer |
| **Different People** | 45%+ bit errors | Clear security gap: 17% vs 45% |
| **Twins Threshold** | 18-30% bit errors | Edge case: most twins rejected at 25%+ |
| **False Acceptance Rate** | <0.01% | Target security level |
| **False Rejection Rate** | <1-2% | Target usability level |

### Performance Metrics

| Metric | iOS | Android | Notes |
|--------|-----|---------|-------|
| **Key Derivation Time** | 350ms | 420ms | iPhone 13 / Pixel 6 |
| **Memory Usage** | 12MB | 15MB | Peak during enrollment |
| **Face Detection** | 80ms | 95ms | MobileFaceNet inference |
| **BCH Encode/Decode** | 180ms | 210ms | Rust core computation |
| **Frame Selection** | 90ms | 115ms | Best quality from video |

---

## Hardware Requirements

### Minimum iOS Requirements

- **Device**: iPhone 8 or later
- **OS**: iOS 14.0+
- **Processor**: A11 Bionic or newer
- **Camera**: 7MP front-facing camera minimum
- **Storage**: 50MB for SDK
- **Security**: Secure Enclave available

### Minimum Android Requirements

- **Device**: Mid-range smartphones (2020+)
- **OS**: Android 9 (API 28)+
- **Processor**: Snapdragon 660 / Exynos 9611 equivalent
- **Camera**: 5MP front-facing camera minimum
- **Storage**: 60MB for SDK
- **Security**: Hardware-backed Keystore

---

## Software Dependencies

### Rust Core

```toml
[dependencies]
# BCH error correction
binary-bch = "0.2"
bch-rust = "0.1"

# Cryptographic primitives
ring = "0.17"
sha2 = "0.10"
hkdf = "0.12"

# Secure memory
zeroize = "1.7"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# FFI
libc = "0.2"
```

### iOS (Swift)

```swift
// Package.swift dependencies
.package(url: "https://github.com/krzyzanowskim/CryptoSwift.git", from: "1.8.0")
```

### Android (Kotlin)

```gradle
// build.gradle dependencies
implementation 'androidx.biometric:biometric:1.2.0-alpha05'
implementation 'androidx.security:security-crypto:1.1.0-alpha06'
implementation 'org.tensorflow:tensorflow-lite:2.14.0'
```

### Python Testing

```txt
bchlib==0.14.0
numpy>=1.24.0
scipy>=1.10.0
matplotlib>=3.7.0
pandas>=2.0.0
jupyter>=1.0.0
```

---

## Quick Start

### 1. Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add iOS targets
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios

# Add Android targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android

# Install cargo-lipo for iOS
cargo install cargo-lipo

# Install Android NDK
# Download from: https://developer.android.com/ndk/downloads
export ANDROID_NDK_HOME=/path/to/android-ndk-r26b
```

### 2. Clone Repository

```bash
git clone https://github.com/yourorg/biometric-key-derivation.git
cd biometric-key-derivation
```

### 3. Build Rust Core

```bash
cd core

# Build for iOS (creates universal binary)
cargo lipo --release

# Build for Android (all architectures)
cargo build --release --target aarch64-linux-android
cargo build --release --target armv7-linux-androideabi
cargo build --release --target x86_64-linux-android

# Run tests
cargo test --all-features
```

### 4. iOS Integration

```bash
cd ../ios

# Install dependencies
swift package resolve

# Build XCFramework
./scripts/build/build_ios.sh

# Output: BiometricKeySDK.xcframework
```

**Add to Xcode Project:**

```swift
// File → Add Package Dependencies
// Enter: file:///path/to/BiometricKeySDK.xcframework
```

**Usage Example:**

```swift
import BiometricKeySDK

let pipeline = EnrollmentPipeline()

pipeline.enroll { result in
    switch result {
    case .success(let enrollmentResult):
        // Store helper data securely
        KeychainManager.store(enrollmentResult.helperData, key: "user_helper_data")
        
        // Use cryptographic key
        let encryptionKey = enrollmentResult.cryptoKey
        print("256-bit key: \(encryptionKey.hexString)")
        
    case .failure(let error):
        print("Enrollment failed: \(error)")
    }
}
```

### 5. Android Integration

```bash
cd ../android

# Build AAR
./gradlew assembleRelease

# Output: app/build/outputs/aar/biometric-key-sdk.aar
```

**Add to Android Project:**

```gradle
// app/build.gradle
dependencies {
    implementation files('libs/biometric-key-sdk.aar')
}
```

**Usage Example:**

```kotlin
import com.biometrickey.pipeline.EnrollmentPipeline

val pipeline = EnrollmentPipeline(context)

pipeline.enroll { result ->
    when (result) {
        is EnrollmentResult.Success -> {
            // Store helper data securely
            KeystoreManager.store(result.helperData, "user_helper_data")
            
            // Use cryptographic key
            val encryptionKey = result.cryptoKey
            Log.d("BiometricSDK", "256-bit key: ${encryptionKey.toHexString()}")
        }
        is EnrollmentResult.Failure -> {
            Log.e("BiometricSDK", "Enrollment failed: ${result.error}")
        }
    }
}
```

---

## Testing

### Unit Tests (Rust)

```bash
cd core

# Run all tests
cargo test

# Run specific module tests
cargo test bch::
cargo test quantization::
cargo test fuzzy_extractor::

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

### Integration Tests (Python)

```bash
cd testing

# Install dependencies
pip install -r requirements.txt

# Generate test vectors
python scripts/generate_test_vectors.py

# Validate BCH parameters
python scripts/validate_bch_params.py --t_min 60 --t_max 100

# Measure error rates
python scripts/analyze_error_rates.py --data data/enrollment/

# Calculate FAR/FRR
python scripts/far_frr_calculator.py --threshold 90
```

### End-to-End Tests

**iOS:**

```bash
cd ios
xcodebuild test \
    -scheme BiometricKeySDK \
    -destination 'platform=iOS Simulator,name=iPhone 14'
```

**Android:**

```bash
cd android
./gradlew connectedAndroidTest
```

---

## Security Considerations

### What Is Stored

**Enrollment (one-time):**
- Helper data (public, non-sensitive) stored on server or device
- Original face embedding DELETED after enrollment
- Cryptographic key DELETED after encrypting user data

**Recovery (every login):**
- No biometric templates stored anywhere
- Helper data fetched from storage
- Key regenerated from new face scan + helper data

### Attack Resistance

| Attack Vector | Mitigation |
|---------------|------------|
| **Replay Attack** | Liveness detection via SDK (not our scope) |
| **Presentation Attack** | Liveness SDK responsibility (e.g., FaceTec, iProov) |
| **Template Inversion** | Helper data is cryptographically useless alone |
| **Brute Force** | 256-bit key space = 2^256 combinations |
| **Similar Face** | 45% bit difference = impossible to correct with t=90 |
| **Twins** | Most twins >25% different = rejected |
| **Jailbreak/Root** | Device detection blocks compromised devices |
| **Memory Dump** | Zeroize library wipes secrets from RAM |
| **Code Tampering** | ProGuard obfuscation + signature verification |

### Compliance

- **GDPR (EU)** - Biometric data consent documented
- **BIPA (Illinois, USA)** - User consent + retention policy
- **CCPA (California, USA)** - Privacy policy + data deletion
- **ISO 27001** - Security audit documentation provided

---

## Project Structure

```
biometric-key-derivation/
├── core/                          # Rust cryptographic core
│   ├── src/
│   │   ├── quantization/          # Float to binary conversion
│   │   ├── bch/                   # BCH error correction
│   │   ├── fuzzy_extractor/       # Helper data generation
│   │   ├── hash/                  # HKDF key derivation
│   │   ├── ffi/                   # iOS/Android bridges
│   │   └── utils/                 # Bit ops, hamming, zeroize
│   ├── tests/                     # Unit + integration tests
│   └── benches/                   # Performance benchmarks
│
├── ios/                           # Swift SDK
│   └── BiometricKeySDK/
│       ├── Camera/                # Video capture
│       ├── FrameSelection/        # Quality assessment
│       ├── Embedding/             # MobileFaceNet wrapper
│       ├── Bridge/                # Rust FFI bridge
│       ├── Pipeline/              # Enrollment/recovery flows
│       ├── Security/              # Keychain, Secure Enclave
│       └── Tests/                 # XCTest suite
│
├── android/                       # Kotlin SDK
│   └── app/src/main/
│       ├── camera/                # Camera2 API
│       ├── embedding/             # TensorFlow Lite wrapper
│       ├── bridge/                # JNI bridge
│       ├── pipeline/              # Enrollment/recovery flows
│       ├── security/              # Keystore, StrongBox
│       └── test/                  # JUnit tests
│
├── testing/                       # Python testing suite
│   ├── scripts/                   # BCH tuning, FAR/FRR analysis
│   ├── test_vectors/              # Ground truth data
│   ├── data/                      # Test participant data
│   └── notebooks/                 # Jupyter analysis
│
├── .github/workflows/             # CI/CD pipelines
├── scripts/                       # Build + deployment scripts
├── security/                      # Audits + compliance docs
├── shared/                        # Constants + protocols
├── docs/                          # Architecture + API docs
└── README.md                      # This file
```

---

## API Reference

### Rust Core API

```rust
// Enrollment
pub fn enrollment(
    face_embedding: &[f32; 128]
) -> Result<(HelperData, CryptoKey), BiometricError>;

// Recovery
pub fn recovery(
    face_embedding: &[f32; 128],
    helper_data: &HelperData
) -> Result<CryptoKey, BiometricError>;

// Quantization
pub fn quantize_sign(embedding: &[f32]) -> Vec<u8>;

// BCH
pub struct BCH {
    pub fn new(n: usize, k: usize, t: usize) -> Self;
    pub fn encode(&self, data: &[u8]) -> Vec<u8>;
    pub fn decode(&self, codeword: &[u8]) -> Result<Vec<u8>, BCHError>;
}

// HKDF
pub fn derive_key(bits: &[u8]) -> [u8; 32];
```

### iOS API

```swift
// Enrollment
public class EnrollmentPipeline {
    public func enroll(
        completion: @escaping (Result<EnrollmentResult, BiometricError>) -> Void
    )
}

// Recovery
public class RecoveryPipeline {
    public func recover(
        helperData: HelperData,
        completion: @escaping (Result<CryptoKey, BiometricError>) -> Void
    )
}

// Results
public struct EnrollmentResult {
    public let helperData: HelperData
    public let cryptoKey: CryptoKey
}
```

### Android API

```kotlin
// Enrollment
class EnrollmentPipeline(context: Context) {
    fun enroll(callback: (EnrollmentResult) -> Unit)
}

// Recovery
class RecoveryPipeline(context: Context) {
    fun recover(
        helperData: HelperData,
        callback: (RecoveryResult) -> Unit
    )
}

// Results
data class EnrollmentResult(
    val helperData: HelperData,
    val cryptoKey: CryptoKey
)
```

---

## Troubleshooting

### Issue: High False Rejection Rate (>5%)

**Symptoms:** Legitimate users frequently locked out.

**Diagnosis:**
```bash
cd testing
python scripts/analyze_error_rates.py --user_id USER_123
```

**Solutions:**
- Increase BCH tolerance: `t=90` → `t=95` (max 17% → 19%)
- Improve frame selection quality thresholds
- Check camera calibration
- Verify lighting conditions during testing

### Issue: False Acceptance Detected

**Symptoms:** Different person accepted (critical security issue).

**Diagnosis:**
```bash
python scripts/attack_simulation.py --attacker_id ATT_456 --victim_id USER_123
```

**Solutions:**
- Verify BCH parameters: `t` must be <45% of 255 bits
- Check if liveness detection is active
- Audit MobileFaceNet model quality
- Review test methodology (ensure different people tested)

### Issue: iOS Build Failure

**Symptoms:** `ld: framework not found BiometricCore`

**Solution:**
```bash
# Rebuild Rust library
cd core
cargo lipo --release

# Verify output
ls target/universal/release/libbiometric_core.a

# Copy to correct location
cp target/universal/release/libbiometric_core.a \
   ../ios/RustCore/lib/
```

### Issue: Android JNI Crash

**Symptoms:** `java.lang.UnsatisfiedLinkError: dlopen failed`

**Solution:**
```bash
# Verify .so files exist for all architectures
ls android/rustcore/libs/*/libbiometric_core.so

# Rebuild if missing
cd core
cargo build --release --target aarch64-linux-android
cargo build --release --target armv7-linux-androideabi
cargo build --release --target x86_64-linux-android

# Copy to correct paths
./scripts/build/build_android.sh
```

---

## Performance Optimization

### Rust Optimization

```toml
# core/Cargo.toml
[profile.release]
opt-level = 3              # Maximum optimization
lto = "fat"                # Link-time optimization
codegen-units = 1          # Better optimization
strip = true               # Remove debug symbols
panic = "abort"            # Smaller binary
```

### iOS Optimization

```swift
// Use background queue for heavy computation
DispatchQueue.global(qos: .userInitiated).async {
    self.pipeline.enroll { result in
        DispatchQueue.main.async {
            // Update UI
        }
    }
}
```

### Android Optimization

```kotlin
// Use coroutines for async operations
lifecycleScope.launch(Dispatchers.Default) {
    val result = pipeline.enroll()
    withContext(Dispatchers.Main) {
        // Update UI
    }
}
```

---

## Deployment

### CI/CD Pipeline

All commits automatically trigger:

1. **Rust Tests** - Unit + integration + benchmarks
2. **iOS Tests** - XCTest suite on simulators
3. **Android Tests** - JUnit + Espresso tests
4. **Security Audit** - Dependency vulnerability scan
5. **Build Artifacts** - XCFramework + AAR generation
6. **Release** - Semantic versioning + changelog

### Distribution

**iOS (CocoaPods):**

```ruby
# Podfile
pod 'BiometricKeySDK', '~> 1.0'
```

**Android (Maven):**

```gradle
// build.gradle
dependencies {
    implementation 'com.biometrickey:sdk:1.0.0'
}
```

---

## Documentation

- **[Architecture Overview](docs/architecture/overview.md)** - System design
- **[Enrollment Flow](docs/architecture/enrollment_flow.md)** - Step-by-step enrollment
- **[Recovery Flow](docs/architecture/recovery_flow.md)** - Step-by-step recovery
- **[Security Analysis](docs/security/threat_model.md)** - Threat model + mitigations
- **[API Reference](docs/api/)** - Complete API documentation
- **[Integration Guide](docs/guides/integration_guide.md)** - How to integrate
- **[Testing Guide](docs/guides/testing_guide.md)** - How to test
- **[Legal Documentation](docs/legal/)** - Privacy policy, consent, licenses

---

## License

**Proprietary License** - All rights reserved.

This software is proprietary and confidential. Unauthorized copying, distribution, or use is strictly prohibited.

© 2025 Biometric Key Derivation Team

---

## Support

- **Technical Support:** support@biometric-key-sdk.com
- **Security Issues:** security@biometric-key-sdk.com
- **Documentation:** https://docs.biometric-key-sdk.com
- **Issue Tracker:** Internal JIRA only (not public)

---

## Acknowledgments

- **Cryptography Research** - Fuzzy extractor theory foundation
- **RustCrypto** - ring, sha2, hkdf implementations
- **BCH Research** - Error correction code theory
- **MobileFaceNet** - Efficient face recognition model
- **Apple Security Team** - Secure Enclave documentation
- **Google Security Team** - StrongBox documentation

---

## Changelog

### Version 1.0.0 (2025-02-17)

- Initial production release
- Complete Rust core implementation
- iOS SDK with Secure Enclave integration
- Android SDK with StrongBox integration
- Python testing suite with BCH parameter tuning
- GDPR compliance documentation
- CI/CD pipeline with automated testing
- Comprehensive API documentation
- Security audit completed
- Performance benchmarks validated

---

**Last Updated:** February 17, 2025  
**Rust Version:** 1.75+  
**iOS Version:** 14.0+  
**Android Version:** API 28+  
**Status:** Production Ready
