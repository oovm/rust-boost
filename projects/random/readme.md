# `random`

Deterministic random streams with a frozen output profile, unbiased sampling, and host entropy backends owned by this crate.

`random` keeps two ownership areas apart and owns both of them:

- Deterministic bytes. `Seed` and `Stream` produce the `boost-chacha12-v1` profile: raw ChaCha12 over a 256-bit key, a 64-bit block counter, and a 64-bit stream identifier. `fill`, `u32`, `u64`, `range`, `unit_f32`, `unit_f64`, `chance`, `index`, `choose`, and `shuffle` share one byte-consumption order, so the same seed, stream identifier, and call sequence always produce the same output.
- Host entropy. `Secure` and `EntropySource` read the platform source directly. The `system` feature compiles the backends owned by this crate instead of delegating to another random crate: `BCryptGenRandom` on Windows, `getrandom` on Linux, and `getentropy` on Apple platforms. A target without an implemented and tested backend reports `ErrorKind::Unsupported` for every non-empty request instead of substituting a weaker source.

`range` accepts `u32`, `u64`, `i32`, and `i64` over `Range` and `RangeInclusive` only, samples with rejection instead of modulo scaling, and rejects invalid bounds without consuming bytes.

`random::distribution` holds the parameter models of the first discrete batch: `UniformU32`, `UniformU64`, `UniformI32`, `UniformI64`, `Bernoulli`, and `Categorical`. A distribution is an immutable parameter object that owns no random state, validates its parameters during construction, and samples through the same kernel as `range` and `chance`, so a stream and an entropy handle cannot drift apart. `Secure` implements `RandomSource` for that reason, and the reverse conversion does not exist. Reported masses are float approximations of the exact integer law they model.

`Stream` and `Secure` expose the same first-batch entries, and each entry is a thin wrapper over one shared implementation. The handles differ only in where the bytes come from: seeded bytes repeat on every target, host entropy does not, and every non-empty entropy request can fail.

Every sampling entry returns `Result`: the profile has a finite capacity, range or probability arguments can be invalid, and a host entropy request can fail. `Secure::system()` only constructs a handle. The first non-empty request probes the platform, an empty request never touches it, and a failed request clears the whole buffer so unmarked partial entropy is never delivered. The handle caches no random bytes, so there is no user-space state for a fork to duplicate and no global registry to keep in sync.

The default build is `no_std` and requires no `alloc`. `std` only adds standard error integration, and `system` adds the platform entropy backends of this crate.
