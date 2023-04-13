//! Deterministic byte sources owned by this crate.
//!
//! An algorithm owns its key shape, nonce, capacity, and byte profile. Nothing here is selected
//! implicitly: a caller names the type and supplies exactly the seed shape that type expects, so a
//! future algorithm does not inherit the 32-byte key, the stream identifier, or the capacity of
//! [`ChaCha12`].

mod chacha12;

pub use chacha12::ChaCha12;
