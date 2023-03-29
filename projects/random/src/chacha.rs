//! Private ChaCha12 core of the `boost-chacha12-v1` profile.

use crate::error::{Error, ErrorKind};

/// ChaCha constants for a 256-bit key: the bytes of `expand 32-byte k` in little-endian words.
const CONSTANTS: [u32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];

/// Size of one ChaCha block in bytes.
pub(crate) const BLOCK_BYTES: usize = 64;

/// Total keystream capacity of the profile: `2^64` blocks of 64 bytes.
pub(crate) const MAX_OUTPUT: u128 = 1u128 << 70;

/// Double rounds performed by the profile, that is 12 rounds.
const DOUBLE_ROUNDS: usize = 6;

/// Raw ChaCha12 keystream byte producer.
///
/// The position is a byte count that cannot wrap: the profile ends after `MAX_OUTPUT` bytes, so
/// `u128` represents both the current position and the end of the stream without a sentinel.
pub(crate) struct ChaChaCore {
    key: [u32; 8],
    nonce: [u32; 2],
    buffer: [u8; BLOCK_BYTES],
    buffered: Option<u64>,
    position: u128,
}

impl ChaChaCore {
    /// Creates a core that uses `key` and the 64-bit `stream_id` as its nonce.
    pub(crate) fn new(key: &[u8; 32], stream_id: u64) -> Self {
        let mut words = [0u32; 8];
        for (word, chunk) in words.iter_mut().zip(key.as_chunks::<4>().0) {
            *word = u32::from_le_bytes(*chunk);
        }
        Self {
            key: words,
            nonce: [stream_id as u32, (stream_id >> 32) as u32],
            buffer: [0u8; BLOCK_BYTES],
            buffered: None,
            position: 0,
        }
    }

    /// Fills `dst` with the next keystream bytes in consumption order.
    ///
    /// A request that would cross the profile capacity fails without advancing the position and
    /// clears `dst`, so a caller never observes bytes from a refused request.
    pub(crate) fn fill(&mut self, dst: &mut [u8]) -> Result<(), Error> {
        if dst.is_empty() {
            return Ok(());
        }
        if dst.len() as u128 > MAX_OUTPUT - self.position {
            dst.fill(0);
            return Err(Error::new(ErrorKind::StreamExhausted));
        }

        let block_bytes = BLOCK_BYTES as u128;
        let mut written = 0;
        while written < dst.len() {
            let offset = (self.position % block_bytes) as usize;
            let index = (self.position / block_bytes) as u64;
            if self.buffered != Some(index) {
                self.buffer = block(&self.key, &self.nonce, index);
                self.buffered = Some(index);
            }
            let take = core::cmp::min(BLOCK_BYTES - offset, dst.len() - written);
            dst[written..written + take].copy_from_slice(&self.buffer[offset..offset + take]);
            written += take;
            self.position += take as u128;
        }
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
