//! Word-size independence fixtures for the sampling entries.
//!
//! §9.2 requires the same representable length to produce the same result on a 32-bit and a 64-bit
//! target, so no entry may pick its algorithm from the native `usize` width. The values below are
//! recorded for one fixed seed and asserted absolutely, which makes them meaningful evidence when
//! this file also runs on a 32-bit target: a `usize`-dependent mapping would fail on one of the two
//! word sizes while a mapping over an explicit `u64` interval passes on both.

use random::{ChaCha12, Random};

/// Seed used by every fixture in this file.
const SEED: u64 = 0x5eed_1234;

/// Stream identifier used by every fixture in this file.
const STREAM_ID: u64 = 9;

/// Creates the fixture source.
fn fixture() -> Random<ChaCha12> {
    Random::new(ChaCha12::from_u64(SEED, STREAM_ID))
}

// Recorded from the profile for the fixture seed, then confirmed unchanged on a 32-bit target.
const GOLDEN_RANGES: [u32; 16] = [4, 2, 5, 4, 3, 4, 5, 3, 5, 2, 2, 5, 6, 6, 2, 2];
const GOLDEN_WIDE: [u64; 8] = [17, 37, 17, 33, 30, 38, 21, 11];
const GOLDEN_SIGNED: [i32; 8] = [2, 1, 2, 2, 2, -1, 2, 4];
const GOLDEN_LONG: [i64; 8] = [
    -8_774_695_867_499_945_274,
    -1_293_633_865_486_821_211,
    -4_783_056_235_797_502_094,
    -9_077_696_624_276_034_071,
    -8_017_756_383_242_014_165,
    -8_549_429_451_412_544_408,
    -2_657_262_421_901_572_364,
    -1_484_254_196_385_424_975,
];
const GOLDEN_INDICES: [usize; 8] = [236_015, 407_165, 846_346, 468_393, 145_747, 600_271, 868_797, 235_873];
const GOLDEN_CHOSEN: [i32; 8] = [0, 90, 0, 90, 90, 0, -90, 90];
const GOLDEN_CHANCES: [bool; 32] = [
    false, false, true, false, false, true, false, false, true, false, false, true, true, false, false, false, true, false,
    false, false, false, false, false, false, false, false, false, false, true, true, true, true,
];
const GOLDEN_SHUFFLED: [u32; 32] =
    [13, 26, 8, 6, 11, 27, 20, 4, 2, 21, 0, 16, 7, 14, 12, 1, 22, 24, 3, 23, 30, 10, 28, 19, 15, 31, 5, 29, 17, 25, 9, 18];

/// Records every fixture value, so a maintainer can refresh the constants after a deliberate
/// profile change. It is ignored because the committed constants are the fixture.
#[test]
#[ignore = "run manually to refresh the word-size goldens"]
fn record_word_size_goldens() {
    let mut random = fixture();

    let ranges: Vec<u32> = (0..16).map(|_| random.range_u32(1..=6).unwrap()).collect();
    let wide: Vec<u64> = (0..8).map(|_| random.range_u64(10..40).unwrap()).collect();
    let signed: Vec<i32> = (0..8).map(|_| random.range_i32(-4..=4).unwrap()).collect();
    let long: Vec<i64> = (0..8).map(|_| random.range_i64(i64::MIN..=-1).unwrap()).collect();
    let indices: Vec<usize> = (0..8).map(|_| random.index(1_000_000).unwrap().unwrap()).collect();
    let chosen: Vec<i32> = (0..8).map(|_| *random.choose(&[-90, 0, 90]).unwrap().unwrap()).collect();
    let chances: Vec<bool> = (0..32).map(|_| random.chance(1, 3).unwrap()).collect();
    let mut items: Vec<u32> = (0..32).collect();
    random.shuffle(&mut items).unwrap();

    println!("ranges = {ranges:?}");
    println!("wide = {wide:?}");
    println!("signed = {signed:?}");
    println!("long = {long:?}");
    println!("indices = {indices:?}");
    println!("chosen = {chosen:?}");
    println!("chances = {chances:?}");
    println!("shuffled = {items:?}");
}

#[test]
fn recorded_entries_are_word_size_independent() {
    let mut random = fixture();

    let ranges: Vec<u32> = (0..16).map(|_| random.range_u32(1..=6).unwrap()).collect();
    let wide: Vec<u64> = (0..8).map(|_| random.range_u64(10..40).unwrap()).collect();
    let signed: Vec<i32> = (0..8).map(|_| random.range_i32(-4..=4).unwrap()).collect();
    let long: Vec<i64> = (0..8).map(|_| random.range_i64(i64::MIN..=-1).unwrap()).collect();
    let indices: Vec<usize> = (0..8).map(|_| random.index(1_000_000).unwrap().unwrap()).collect();
    let chosen: Vec<i32> = (0..8).map(|_| *random.choose(&[-90, 0, 90]).unwrap().unwrap()).collect();
    let chances: Vec<bool> = (0..32).map(|_| random.chance(1, 3).unwrap()).collect();
    let mut items: Vec<u32> = (0..32).collect();
    random.shuffle(&mut items).unwrap();

    assert_eq!(ranges, GOLDEN_RANGES);
    assert_eq!(wide, GOLDEN_WIDE);
    assert_eq!(signed, GOLDEN_SIGNED);
    assert_eq!(long, GOLDEN_LONG);
    assert_eq!(indices, GOLDEN_INDICES);
    assert_eq!(chosen, GOLDEN_CHOSEN);
    assert_eq!(chances, GOLDEN_CHANCES);
    assert_eq!(items, GOLDEN_SHUFFLED);
}

#[test]
fn a_maximal_usize_length_stays_inside_its_own_bound() {
    // `usize::MAX` is a different length on a 32-bit and a 64-bit target, so the sampled index is
    // intentionally different and cannot be one fixture. What must not differ is the mapping path:
    // the length is converted to `u64`, sampled as a `u64` interval, and converted back, so a
    // 32-bit target never falls back to a narrower algorithm for the same request.
    let mut random = fixture();
    let index = random.index(usize::MAX).unwrap().unwrap();
    assert!(index < usize::MAX, "an index at the maximal length left its bound: {index}");
}
