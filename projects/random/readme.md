# `random`

Deterministic random streams with a frozen output profile and unbiased sampling.

`random` separates two ownership areas: deterministic bytes produced from an explicit seed, and later acquisition of host entropy. This release implements the deterministic half only.

- `Seed` is a 32-byte key with an explicit encoding and a redacted `Debug`.
- `Stream::new(seed, stream_id)` produces the `boost-chacha12-v1` profile: raw ChaCha12 over a 256-bit key, a 64-bit block counter, and a 64-bit stream identifier.
- `fill`, `u32`, `u64`, `range`, `unit_f32`, `unit_f64`, `chance`, `index`, `choose`, and `shuffle` share one byte-consumption order, so the same seed, stream identifier, and call sequence always produce the same output.
- `range` accepts `u32`, `u64`, `i32`, and `i64` over `Range` and `RangeInclusive` only, uses rejection sampling instead of modulo scaling, and rejects invalid bounds without consuming bytes.

Every sampling entry returns `Result`: the profile has a finite capacity, and range or probability arguments can be invalid. The default build is `no_std` and does not require `alloc`; the `std` feature only adds standard error integration.

Host entropy backends are not part of this release, so `Secure` does not exist yet. System entropy must eventually be owned by this crate rather than delegated to another random crate.
