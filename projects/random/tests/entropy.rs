//! Contract tests for the host entropy surface.
//!
//! The crate tests observe providers through the public API only: a recording provider counts how
//! often it was asked for bytes, and a failing provider writes a partial prefix before reporting
//! the failure, so the clearing contract is verified from the outside.

use core::cell::Cell;

use random::{ChaCha12, EntropyError, EntropySource, Random, Secure, SourceError};

/// Byte pattern produced by the recording provider.
const PATTERN: u8 = 0x42;

/// Provider that records how often it was asked for bytes.
struct RecordingProvider<'a> {
    calls: &'a Cell<usize>,
}

impl EntropySource for RecordingProvider<'_> {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), EntropyError> {
        self.calls.set(self.calls.get() + 1);
        dst.fill(PATTERN);
        Ok(())
    }
}

/// Provider that writes a partial prefix and then fails.
struct FailingProvider;

impl EntropySource for FailingProvider {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), EntropyError> {
        let prefix = dst.len() / 2;
        dst[..prefix].fill(0xAA);
        Err(EntropyError::Failed)
    }
}

/// Provider that fails while preserving a raw platform code.
struct CodedProvider;

impl EntropySource for CodedProvider {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), EntropyError> {
        dst.fill(0xAA);
        Err(EntropyError::Errno(11))
    }
}

/// Provider that reports a platform without a usable backend.
struct UnsupportedProvider;

impl EntropySource for UnsupportedProvider {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), EntropyError> {
        dst.fill(0xAA);
        Err(EntropyError::Unsupported)
    }
}

#[test]
fn empty_request_never_reaches_the_provider() {
    let calls = Cell::new(0);
    let mut secure = Secure::with_entropy(RecordingProvider { calls: &calls });
    secure.fill(&mut []).unwrap();
    assert_eq!(calls.get(), 0);

    let mut bytes = [0u8; 4];
    secure.fill(&mut bytes).unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(bytes, [PATTERN; 4]);
}

#[test]
fn failed_request_clears_the_whole_buffer() {
    let mut secure = Secure::with_entropy(FailingProvider);
    let mut bytes = [0x11u8; 16];
    assert_eq!(secure.fill(&mut bytes).unwrap_err(), SourceError::Entropy(EntropyError::Failed));
    assert_eq!(bytes, [0u8; 16]);
}

#[test]
fn provider_error_keeps_its_errno_domain() {
    let mut secure = Secure::with_entropy(CodedProvider);
    let mut bytes = [0x11u8; 8];
    let error = secure.fill(&mut bytes).unwrap_err();
    assert_eq!(error, SourceError::Entropy(EntropyError::Errno(11)));
    assert_eq!(bytes, [0u8; 8]);
}

#[test]
fn unsupported_provider_is_reported_without_partial_output() {
    let mut secure = Secure::with_entropy(UnsupportedProvider);
    let mut bytes = [0x11u8; 8];
    let error = secure.fill(&mut bytes).unwrap_err();
    assert_eq!(error, SourceError::Entropy(EntropyError::Unsupported));
    assert_eq!(bytes, [0u8; 8]);
}

#[test]
fn seed_generation_reads_only_the_passed_provider() {
    let calls = Cell::new(0);
    let mut secure = Secure::with_entropy(RecordingProvider { calls: &calls });
    let mut seed = [0u8; 32];
    secure.fill(&mut seed).unwrap();
    assert_eq!(calls.get(), 1);

    let mut from_generated = Random::new(ChaCha12::from_seed(seed, 0));
    let mut from_pattern = Random::new(ChaCha12::from_seed([PATTERN; 32], 0));
    assert_eq!(from_generated.u64().unwrap(), from_pattern.u64().unwrap());
}

#[test]
fn seed_generation_reports_a_failing_provider() {
    let mut secure = Secure::with_entropy(FailingProvider);
    let mut seed = [0u8; 32];
    assert_eq!(secure.fill(&mut seed).unwrap_err(), SourceError::Entropy(EntropyError::Failed));
}

/// Platform backends compiled by this crate, which the test host must exercise directly.
///
/// The `system` feature is not a default, so this module only compiles when a backend exists.
#[cfg(all(feature = "system", any(windows, target_os = "linux", target_os = "wasi", target_vendor = "apple")))]
mod host {
    use random::Secure;

    #[test]
    fn system_entropy_fills_and_varies() {
        let mut secure = Secure::system();
        let mut first = [0u8; 32];
        let mut second = [0u8; 32];
        secure.fill(&mut first).unwrap();
        secure.fill(&mut second).unwrap();
        assert_ne!(first, [0u8; 32]);
        assert_ne!(second, [0u8; 32]);
        assert_ne!(first, second);
    }

    #[test]
    fn system_entropy_fills_a_large_buffer_in_chunks() {
        let mut secure = Secure::system();
        let mut bytes = vec![0u8; 65_536];
        secure.fill(&mut bytes).unwrap();
        assert!(bytes.iter().any(|byte| *byte != 0));
    }

    #[test]
    fn system_entropy_generates_distinct_seeds() {
        let mut secure = Secure::system();
        let mut first = [0u8; 32];
        let mut second = [0u8; 32];
        secure.fill(&mut first).unwrap();
        secure.fill(&mut second).unwrap();
        assert_ne!(first, [0u8; 32]);
        assert_ne!(first, second);
    }

    #[test]
    fn an_empty_request_never_touches_the_host() {
        Secure::system().fill(&mut []).unwrap();
    }
}

/// The unconfigured handle takes the same path as a target without a backend, so the host exercises
/// the `Unsupported` contract as long as the `system` feature is off.
#[cfg(not(feature = "system"))]
mod unconfigured {
    use random::{EntropyError, Secure, SourceError};

    #[test]
    fn system_without_the_feature_reports_unsupported() {
        let mut secure = Secure::system();
        let mut bytes = [0x11u8; 8];
        let error = secure.fill(&mut bytes).unwrap_err();
        assert_eq!(error, SourceError::Entropy(EntropyError::Unsupported));
        assert_eq!(bytes, [0u8; 8]);
    }

    #[test]
    fn an_empty_request_still_succeeds() {
        Secure::system().fill(&mut []).unwrap();
    }
}
