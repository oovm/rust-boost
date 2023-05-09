//! The `boost-chacha12-v1` deterministic byte source.
//!
//! The algorithm is raw ChaCha12 in the original state layout: a 256-bit key, a 64-bit block
//! counter, and a 64-bit nonce, with 64-byte blocks. It is not the IETF 32-bit-counter layout and it
//! is not ChaCha20. ChaCha words are encoded little-endian, and the profile performs 12 rounds, that
//! is 6 double rounds.

use crate::{error::SourceError, source::RandomSource};

/// ChaCha constants for a 256-bit key: the bytes of `expand 32-byte k` in little-endian words.
const CONSTANTS: [u32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];

/// Size of one ChaCha block in bytes.
const BLOCK_BYTES: usize = 64;

/// Total keystream capacity of the profile: `2^64` blocks of 64 bytes.
const MAX_OUTPUT: u128 = 1u128 << 70;

/// Double rounds performed by the profile, that is 12 rounds.
const DOUBLE_ROUNDS: usize = 6;

/// Deterministic byte source of the `boost-chacha12-v1` profile.
///
/// The type owns its key, nonce, counter, capacity, and byte profile. It never reads host entropy,
/// never depends on thread scheduling or target word size, and consumes its state through
/// `&mut self` only. Sampling is not part of this type: a caller samples through
/// [`Random`](crate::Random) or a distribution object, both of which accept any
/// [`RandomSource`].
pub struct ChaCha12 {
    core: ChaChaCore,
}

impl ChaCha12 {
    /// Frozen output profile implemented by this type.
    ///
    /// The profile fixes the algorithm, the 32-byte key encoding, the byte consumption order, the
    /// integer sampling mapping, the float construction, and the shuffle order as one contract while
    /// it is composed with the sampling layer. It identifies the output semantics, not the semver of
    /// the crate.
    pub const PROFILE: &str = "boost-chacha12-v1";

    /// Creates the source from its exact 32-byte key and a 64-bit `stream_id`.
    ///
    /// The key is the ChaCha key and `stream_id` is the nonce. A caller that needs an unpredictable
    /// key must fill the buffer from [`Secure`](crate::Secure) first: a key literal or a counter is
    /// not a secret.
    pub fn from_seed(key: [u8; 32], stream_id: u64) -> Self {
        Self { core: ChaChaCore::new(&key, stream_id) }
    }

    /// Creates the source from a 64-bit `value` and a 64-bit `stream_id`.
    ///
    /// The key is the little-endian encoding of `value` in the first 8 bytes followed by 24 zero
    /// bytes. That spans at most 64 bits and expands no entropy, so this constructor is for tests,
    /// layout, and replay rather than for secrets.
    pub fn from_u64(value: u64, stream_id: u64) -> Self {
        Self::from_seed(seed_from_u64(value), stream_id)
    }
}

impl RandomSource for ChaCha12 {
    /// Fills `dst` with the next bytes of the stream.
    ///
    /// A request longer than the remaining capacity fails with [`SourceError::Exhausted`] without
    /// advancing the stream and clears `dst`, so the source never wraps around silently.
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), SourceError> {
        self.core.fill(dst)
    }
}

/// Encodes a 64-bit value as the 32-byte key of the profile.
const fn seed_from_u64(value: u64) -> [u8; 32] {
    let mut key = [0u8; 32];
    let encoded = value.to_le_bytes();
    let mut index = 0;
    while index < 8 {
        key[index] = encoded[index];
        index += 1;
    }
    key
}

/// Raw ChaCha12 keystream byte producer.
///
/// The stream is tracked as a block index, a byte offset inside the current block, and the bytes
/// left before `MAX_OUTPUT`. Keeping those apart means a fill never divides the position into a
/// block index, which used to cost a `u128` division per delivered chunk.
struct ChaChaCore {
    key: [u32; 8],
    nonce: [u32; 2],
    buffer: [u8; BLOCK_BYTES],
    /// Bytes of `buffer` already delivered; `BLOCK_BYTES` means the buffer must be refilled.
    offset: usize,
    /// The next block to materialize.
    block_index: u64,
    /// Bytes the profile can still deliver.
    remaining: u128,
}

impl ChaChaCore {
    /// Creates a core that uses `key` and the 64-bit `stream_id` as its nonce.
    fn new(key: &[u8; 32], stream_id: u64) -> Self {
        let mut words = [0u32; 8];
        for (word, chunk) in words.iter_mut().zip(key.as_chunks::<4>().0) {
            *word = u32::from_le_bytes(*chunk);
        }
        Self {
            key: words,
            nonce: [stream_id as u32, (stream_id >> 32) as u32],
            buffer: [0u8; BLOCK_BYTES],
            offset: BLOCK_BYTES,
            block_index: 0,
            remaining: MAX_OUTPUT,
        }
    }

    /// Fills `dst` with the next keystream bytes in consumption order.
    ///
    /// A request that would cross the profile capacity fails without advancing the position and
    /// clears `dst`, so a caller never observes bytes from a refused request.
    fn fill(&mut self, dst: &mut [u8]) -> Result<(), SourceError> {
        if dst.is_empty() {
            return Ok(());
        }
        if dst.len() as u128 > self.remaining {
            dst.fill(0);
            return Err(SourceError::Exhausted);
        }

        let mut written = 0;
        while written < dst.len() {
            if self.offset == BLOCK_BYTES {
                self.buffer = block(&self.key, &self.nonce, self.block_index);
                // The profile holds exactly `2^64` blocks, so the index saturates only after the
                // last block, where `remaining` is already zero.
                self.block_index = self.block_index.saturating_add(1);
                self.offset = 0;
            }
            let take = core::cmp::min(BLOCK_BYTES - self.offset, dst.len() - written);
            dst[written..written + take].copy_from_slice(&self.buffer[self.offset..self.offset + take]);
            written += take;
            self.offset += take;
        }
        self.remaining -= written as u128;
        Ok(())
    }
}

/// Computes the block at `counter` for the given key and nonce.
fn block(key: &[u32; 8], nonce: &[u32; 2], counter: u64) -> [u8; BLOCK_BYTES] {
    let mut state = [0u32; 16];
    state[..4].copy_from_slice(&CONSTANTS);
    state[4..12].copy_from_slice(key);
    state[12] = counter as u32;
    state[13] = (counter >> 32) as u32;
    state[14] = nonce[0];
    state[15] = nonce[1];

    let mut working = state;
    for _ in 0..DOUBLE_ROUNDS {
        double_round(&mut working);
    }

    let mut output = [0u8; BLOCK_BYTES];
    for (index, (word, original)) in working.iter().zip(state.iter()).enumerate() {
        let sum = word.wrapping_add(*original);
        output[index * 4..index * 4 + 4].copy_from_slice(&sum.to_le_bytes());
    }
    output
}

/// Applies one column round followed by one diagonal round.
fn double_round(state: &mut [u32; 16]) {
    quarter_round(state, 0, 4, 8, 12);
    quarter_round(state, 1, 5, 9, 13);
    quarter_round(state, 2, 6, 10, 14);
    quarter_round(state, 3, 7, 11, 15);
    quarter_round(state, 0, 5, 10, 15);
    quarter_round(state, 1, 6, 11, 12);
    quarter_round(state, 2, 7, 8, 13);
    quarter_round(state, 3, 4, 9, 14);
}

/// Applies one ChaCha quarter round in place.
fn quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    state[a] = state[a].wrapping_add(state[b]);
    state[d] = (state[d] ^ state[a]).rotate_left(16);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] = (state[b] ^ state[c]).rotate_left(12);
    state[a] = state[a].wrapping_add(state[b]);
    state[d] = (state[d] ^ state[a]).rotate_left(8);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] = (state[b] ^ state[c]).rotate_left(7);
}
