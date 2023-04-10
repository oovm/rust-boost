//! Throughput comparison against the comparison crates, measured on this host only.
//!
//! The benchmarks use the nightly test harness instead of adding a benchmark framework, so the
//! crate's dependency graph stays what it is: `rand` and `fastrand` are dev-dependencies of this
//! package and never appear in a shipped build. Numbers from this file describe one CPU, one target,
//! and one feature set, which is exactly why the contract does not let them become a "faster than"
//! claim on their own.

#![feature(test)]

extern crate test;

use random::{Secure, Seed, Stream};
use test::Bencher;

/// Reads 64 bytes from the deterministic stream.
#[bench]
fn stream_fill_64(b: &mut Bencher) {
    let mut stream = Stream::new(Seed::from_u64(1), 0);
    let mut buffer = [0u8; 64];
    b.iter(|| stream.fill(&mut buffer).unwrap());
}

/// Reads 64 bytes from `rand`, the comparison crate.
#[bench]
fn rand_fill_64(b: &mut Bencher) {
    use rand::{RngCore, SeedableRng};

    let mut rng = rand::rngs::StdRng::seed_from_u64(1);
    let mut buffer = [0u8; 64];
    b.iter(|| rng.fill_bytes(&mut buffer));
}

/// Reads 64 bytes from `fastrand`, the comparison crate.
#[bench]
fn fastrand_fill_64(b: &mut Bencher) {
    let mut rng = fastrand::Rng::with_seed(1);
    let mut buffer = [0u8; 64];
    b.iter(|| rng.fill(&mut buffer));
}

/// Draws one `u32` word from the deterministic stream.
#[bench]
fn stream_word(b: &mut Bencher) {
    let mut stream = Stream::new(Seed::from_u64(2), 0);
    b.iter(|| stream.u32().unwrap());
}

/// Draws one `u32` word from the comparison crate.
#[bench]
fn rand_word(b: &mut Bencher) {
    use rand::{Rng, SeedableRng};

    let mut rng = rand::rngs::StdRng::seed_from_u64(2);
    b.iter(|| rng.r#gen::<u32>());
}

/// Draws one `u32` word from `fastrand`.
#[bench]
fn fastrand_word(b: &mut Bencher) {
    let mut rng = fastrand::Rng::with_seed(2);
    b.iter(|| rng.u32(..));
}

/// Draws one value from a small non-power-of-two interval, which exercises rejection sampling.
#[bench]
fn stream_range_1_to_6(b: &mut Bencher) {
    let mut stream = Stream::new(Seed::from_u64(3), 0);
    b.iter(|| stream.range(1u32..=6).unwrap());
}

/// Draws one value from the same interval through the comparison crate.
#[bench]
fn rand_range_1_to_6(b: &mut Bencher) {
    use rand::{Rng, SeedableRng};

    let mut rng = rand::rngs::StdRng::seed_from_u64(3);
    b.iter(|| rng.gen_range(1u32..=6));
}

/// Draws one value from the same interval through `fastrand`.
#[bench]
fn fastrand_range_1_to_6(b: &mut Bencher) {
    let mut rng = fastrand::Rng::with_seed(3);
    b.iter(|| rng.u32(1..=6));
}

/// Shuffles 32 elements through the deterministic stream.
#[bench]
fn stream_shuffle_32(b: &mut Bencher) {
    let mut stream = Stream::new(Seed::from_u64(4), 0);
    let mut items: Vec<u32> = (0..32).collect();
    b.iter(|| stream.shuffle(&mut items).unwrap());
}

/// Shuffles 32 elements through the comparison crate.
#[bench]
fn rand_shuffle_32(b: &mut Bencher) {
    use rand::{SeedableRng, seq::SliceRandom};

    let mut rng = rand::rngs::StdRng::seed_from_u64(4);
    let mut items: Vec<u32> = (0..32).collect();
    b.iter(|| items.shuffle(&mut rng));
}

/// Shuffles 32 elements through `fastrand`.
#[bench]
fn fastrand_shuffle_32(b: &mut Bencher) {
    let mut rng = fastrand::Rng::with_seed(4);
    let mut items: Vec<u32> = (0..32).collect();
    b.iter(|| rng.shuffle(&mut items));
}

/// Requests 32 bytes from the host entropy source owned by this crate.
///
/// There is no comparison entry: the comparison crates reach the host through their own dependency,
/// which this crate deliberately does not link, so the two numbers would describe different code.
#[bench]
fn host_entropy_32(b: &mut Bencher) {
    let mut secure = Secure::system();
    let mut buffer = [0u8; 32];
    b.iter(|| secure.fill(&mut buffer).unwrap());
}

/// Requests 65,536 bytes from the host entropy source, crossing the chunk boundary of every backend.
#[bench]
fn host_entropy_65536(b: &mut Bencher) {
    let mut secure = Secure::system();
    let mut buffer = vec![0u8; 65_536];
    b.iter(|| secure.fill(&mut buffer).unwrap());
}
