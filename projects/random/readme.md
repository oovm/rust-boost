# `random`

Deterministic random streams with a frozen output profile, unbiased sampling, and host entropy backends owned by this crate.

`random` keeps two ownership areas apart and owns both of them:

- Deterministic bytes. `algorithms::ChaCha12` produces the `boost-chacha12-v1` profile: raw ChaCha12 over a 256-bit key, a 64-bit block counter, and a 64-bit stream identifier. It is constructed from an exact 32-byte key with `ChaCha12::from_seed(key, stream_id)`, or from a 64-bit value with `ChaCha12::from_u64(value, stream_id)`. A caller that needs an unpredictable key fills the buffer from `Secure` first, because a key literal or a counter spans little entropy.
- Host entropy. `Secure` and `EntropySource` read the platform source directly. The `system` feature compiles the backends owned by this crate instead of delegating to another random crate: `BCryptGenRandom` on Windows, `getrandom` on Linux, `getentropy` on Apple platforms, and the preview 1 `random_get` import on WASI. The separate `web` feature adds the Web Crypto bridge for `wasm32-unknown-unknown`, probes the host at run time instead of guessing from the target triple, and never reaches a native build, because both the module and its dependency are target-gated. A target without an implemented and tested backend reports `EntropyError::Unsupported` for every non-empty request instead of substituting a weaker source.

Sampling lives in one facade. `Random::new(source)` owns any byte source and exposes `fill`, `u32`, `u64`, `unit_f32`, `unit_f64`, `range_u32`, `range_u64`, `range_i32`, `range_i64`, `chance`, `index`, `choose`, and `shuffle`, all over one byte-consumption order. The four range entries accept standard `RangeBounds`, normalize them during construction, sample with rejection instead of modulo scaling, and reject invalid bounds without consuming bytes.

`Random` owns no algorithm of its own, so the same entries drive a deterministic `ChaCha12` and a host `Secure` handle. `Secure` implements `RandomSource` and `EntropySource`; the reverse conversion from an arbitrary `RandomSource` to an entropy source does not exist. A distribution object can also consume any `RandomSource` directly, so `Random` is convenient rather than mandatory.

`random::distribution` holds the parameter models. A distribution is an immutable parameter object that owns no random state, validates its parameters during construction, and samples through the same kernel as the range and chance entries, so a deterministic source and an entropy handle cannot drift apart.

The discrete batch is `Uniform<u32>`, `Uniform<u64>`, `Uniform<i32>`, `Uniform<i64>`, `Bernoulli`, and `Categorical`, all with an integer law and a float `mass` query. The continuous batch is `UniformF32`, `UniformF64`, `Normal`, and `Exponential`, with `density` and, where the format can express it, a cumulative query.

The distributions differ in how far their output can be reproduced:

- `UniformF32` and `UniformF64` use IEEE arithmetic only, so they are bit-reproducible on every IEEE 754 target. A value that would round up to the excluded upper bound moves one step below it, so the support is never left and no clamp covers a mass shift.
- `Normal` and `Exponential` need a logarithm, a square root, or an exponential. This crate implements those in its own `math` module instead of linking a platform math library, so a distributed build does not depend on the last bits of a system `libm`, and the result carries a stated accuracy bound rather than a bit-for-bit promise.

Every sampling entry returns `Result`. Errors are layered by origin: `EntropyError` is the only layer that carries operating-system, WASI, and Web host differences (`Errno`, `NtStatus`, `Wasi`, `Web`), `SourceError` reports a source failure and wraps a host `EntropyError` as `SourceError::Entropy`, and `RandomError` adds the parameter and numerical variants or propagates a source failure as `RandomError::Source` unchanged. A failure keeps its domain instead of being flattened into one platform code.

`Secure::system()` only constructs a handle. The first non-empty request probes the platform, an empty request never touches it, and a failed request clears the whole buffer so unmarked partial entropy is never delivered. A deterministic source with a finite capacity reports `SourceError::Exhausted` instead of wrapping its position.

The default build is `no_std` and requires no `alloc`. `std` only adds standard error integration, and `system` adds the platform entropy backends of this crate.
