# `random`

Deterministic random streams with a frozen output profile, unbiased sampling, and host entropy backends owned by this crate.

`random` keeps two ownership areas apart and owns both of them:

- Deterministic bytes. `Seed` and `Stream` produce the `boost-chacha12-v1` profile: raw ChaCha12 over a 256-bit key, a 64-bit block counter, and a 64-bit stream identifier. `fill`, `u32`, `u64`, `range`, `unit_f32`, `unit_f64`, `chance`, `index`, `choose`, and `shuffle` share one byte-consumption order, so the same seed, stream identifier, and call sequence always produce the same output.
- Host entropy. `Secure` and `EntropySource` read the platform source directly. The `system` feature compiles the backends owned by this crate instead of delegating to another random crate: `BCryptGenRandom` on Windows, `getrandom` on Linux, `getentropy` on Apple platforms, and the preview 1 `random_get` import on WASI. The separate `web` feature adds the Web Crypto bridge for `wasm32-unknown-unknown`, probes the host at run time instead of guessing from the target triple, and never reaches a native build, because both the module and its dependency are target-gated. A target without an implemented and tested backend reports `ErrorKind::Unsupported` for every non-empty request instead of substituting a weaker source.

`range` accepts `u32`, `u64`, `i32`, and `i64` over `Range` and `RangeInclusive` only, samples with rejection instead of modulo scaling, and rejects invalid bounds without consuming bytes.

`random::distribution` holds the parameter models. A distribution is an immutable parameter object that owns no random state, validates its parameters during construction, and samples through the same kernel as `range` and `chance`, so a stream and an entropy handle cannot drift apart. `Secure` implements `RandomSource` for that reason, and the reverse conversion does not exist.

The discrete batch is `UniformU32`, `UniformU64`, `UniformI32`, `UniformI64`, `Bernoulli`, and `Categorical`, all with an integer law and a float `mass` query. The continuous batch is `UniformF32`, `UniformF64`, `Normal`, and `Exponential`, with `density` and, where the format can express it, a cumulative query.

The distributions differ in how far their output can be reproduced:

- `UniformF32` and `UniformF64` use IEEE arithmetic only, so they are bit-reproducible on every IEEE 754 target. A value that would round up to the excluded upper bound moves one step below it, so the support is never left and no clamp covers a mass shift.
- `Normal` and `Exponential` need a logarithm, a square root, or an exponential. This crate implements those in `math` instead of linking a platform math library, so a distributed build does not depend on the last bits of a system `libm`, and the result carries a stated accuracy bound rather than a bit-for-bit promise.

`Stream` and `Secure` expose the same first-batch entries, and each entry is a thin wrapper over one shared implementation. The handles differ only in where the bytes come from: seeded bytes repeat on every target, host entropy does not, and every non-empty entropy request can fail.

Every sampling entry returns `Result`: the profile has a finite capacity, range or probability arguments can be invalid, and a host entropy request can fail. `Secure::system()` only constructs a handle. The first non-empty request probes the platform, an empty request never touches it, and a failed request clears the whole buffer so unmarked partial entropy is never delivered. The handle caches no random bytes, so there is no user-space state for a fork to duplicate and no global registry to keep in sync.

The default build is `no_std` and requires no `alloc`. `std` only adds standard error integration, and `system` adds the platform entropy backends of this crate.
