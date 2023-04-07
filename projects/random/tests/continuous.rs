//! Contract tests for the continuous uniform distributions.
//!
//! The float interval is a real interval, so the tests check the support boundary, the density and
//! CDF queries, and the two numerical cases the mapping has to survive without hiding a mass shift:
//! an interval whose width is not representable, and an interval narrower than one representable
//! step. A caller-programmed word source supplies the extreme values of the uniform input, so the
//! excluded upper bound is checked exactly instead of statistically.

use random::{
    Error, ErrorKind, RandomSource, Seed, Stream,
    distribution::{UniformF32, UniformF64},
};

/// Deterministic source that repeats one programmed word.
struct WordQueue {
    word: Vec<u8>,
}

impl WordQueue {
    /// Programs an endless sequence of one repeated `u64` word.
    fn repeated_u64(word: u64) -> Self {
        Self { word: word.to_le_bytes().to_vec() }
    }

    /// Programs an endless sequence of one repeated `u32` word.
    fn repeated_u32(word: u32) -> Self {
        Self { word: word.to_le_bytes().to_vec() }
    }
}

impl RandomSource for WordQueue {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Error> {
        if dst.len() > self.word.len() {
            return Err(Error::new(ErrorKind::StreamExhausted));
        }
        dst.copy_from_slice(&self.word[..dst.len()]);
        Ok(())
    }
}

/// Source that plays a programmed sequence of whole words.
struct SequenceQueue {
    bytes: Vec<u8>,
    position: usize,
}

impl SequenceQueue {
    /// Programs a sequence of little-endian `u64` words.
    fn of_u64(words: &[u64]) -> Self {
        let bytes = words.iter().flat_map(|word| word.to_le_bytes()).collect();
        Self { bytes, position: 0 }
    }
}

impl RandomSource for SequenceQueue {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Error> {
        let end = self.position + dst.len();
        if end > self.bytes.len() {
            return Err(Error::new(ErrorKind::StreamExhausted));
        }
        dst.copy_from_slice(&self.bytes[self.position..end]);
        self.position = end;
        Ok(())
    }
}

/// Source that always fails.
struct FailingSource;

impl RandomSource for FailingSource {
    fn try_fill_bytes(&mut self, _dst: &mut [u8]) -> Result<(), Error> {
        Err(Error::new(ErrorKind::BackendFailure))
    }
}

/// Stream used by tests that only need a stable identity.
fn fresh() -> Stream {
    Stream::new(Seed::from_u64(0x5eed), 3)
}

#[test]
fn invalid_parameters_are_rejected() {
    assert_eq!(UniformF64::new(f64::NAN, 1.0).unwrap_err().kind(), ErrorKind::InvalidParameter);
    assert_eq!(UniformF64::new(0.0, f64::NAN).unwrap_err().kind(), ErrorKind::InvalidParameter);
    assert_eq!(UniformF64::new(f64::NEG_INFINITY, 1.0).unwrap_err().kind(), ErrorKind::InvalidParameter);
    assert_eq!(UniformF64::new(0.0, f64::INFINITY).unwrap_err().kind(), ErrorKind::InvalidParameter);
    assert_eq!(UniformF64::new(1.0, 1.0).unwrap_err().kind(), ErrorKind::InvalidParameter);
    assert_eq!(UniformF64::new(2.0, 1.0).unwrap_err().kind(), ErrorKind::InvalidParameter);

    let interval = UniformF64::new(-1.0, 2.5).unwrap();
    assert_eq!(interval.low(), -1.0);
    assert_eq!(interval.high(), 2.5);
}

#[test]
fn samples_stay_inside_the_support() {
    let intervals = [
        (0.0f64, 1.0f64),
        (-1.0, 1.0),
        (-2.5, -1.5),
        (1.0e-300, 1.0e300),
        (-f64::MAX, f64::MAX),
        (f64::MIN_POSITIVE, f64::MIN_POSITIVE * 4.0),
        (1.0, 1.0 + f64::EPSILON),
    ];
    let mut stream = fresh();
    for (low, high) in intervals {
        let interval = UniformF64::new(low, high).unwrap();
        for _ in 0..256 {
            let value = interval.sample(&mut stream).unwrap();
            assert!(value.is_finite(), "interval [{low}, {high}) produced {value}");
            assert!(value >= low && value < high, "interval [{low}, {high}) produced {value}");
        }
    }
}

#[test]
fn the_excluded_upper_bound_keeps_zero_mass() {
    // The largest uniform input must not reach the excluded bound, and the zero input must produce
    // the lower bound exactly.
    for (low, high) in [(0.0f64, 1.0f64), (-1.0, 2.5), (-1.0e300, 1.0e300), (-f64::MAX, f64::MAX)] {
        let interval = UniformF64::new(low, high).unwrap();

        let value = interval.sample(&mut WordQueue::repeated_u64(u64::MAX)).unwrap();
        assert!(value < high, "interval [{low}, {high}) reached its upper bound with {value}");
        assert!(value >= low, "interval [{low}, {high}) left its support with {value}");

        let value = interval.sample(&mut WordQueue::repeated_u64(0)).unwrap();
        assert_eq!(value, low, "the zero input did not produce the lower bound for [{low}, {high})");
    }

    let interval = UniformF32::new(0.0f32, 1.0f32).unwrap();
    let value = interval.sample(&mut WordQueue::repeated_u32(u32::MAX)).unwrap();
    assert!(value < 1.0, "the f32 interval reached its upper bound with {value}");
    assert_eq!(interval.sample(&mut WordQueue::repeated_u32(0)).unwrap(), 0.0);
}

#[test]
fn an_interval_narrower_than_one_step_concentrates_its_mass_inside() {
    // Only one representable value lies inside `[1.0, 1.0 + EPSILON)`, so every sample has to be that
    // value: rounding up to the excluded bound is corrected, and it is not a clamp across the support.
    let interval = UniformF64::new(1.0, 1.0 + f64::EPSILON).unwrap();
    let mut stream = fresh();
    for _ in 0..256 {
        assert_eq!(interval.sample(&mut stream).unwrap(), 1.0);
    }

    let smallest = f64::from_bits(5);
    let interval = UniformF64::new(0.0, smallest).unwrap();
    let mut stream = fresh();
    for _ in 0..256 {
        let value = interval.sample(&mut stream).unwrap();
        assert!(value.is_finite() && value >= 0.0 && value < smallest, "subnormal width produced {value}");
    }
}

#[test]
fn density_and_cdf_describe_the_interval() {
    let unit = UniformF64::new(0.0, 1.0).unwrap();
    assert_eq!(unit.density(), 1.0);
    assert_eq!(unit.cdf(-1.0), 0.0);
    assert_eq!(unit.cdf(0.0), 0.0);
    assert_eq!(unit.cdf(0.25), 0.25);
    assert_eq!(unit.cdf(0.5), 0.5);
    assert_eq!(unit.cdf(1.0), 1.0);
    assert_eq!(unit.cdf(2.0), 1.0);

    let wide = UniformF64::new(-f64::MAX, f64::MAX).unwrap();
    // The width of this interval is not representable, so both queries are computed from half of it.
    assert!(wide.density() > 0.0, "the density of a wide interval must not underflow to zero");
    assert_eq!(wide.density(), 0.5 / f64::MAX);
    assert_eq!(wide.cdf(0.0), 0.5);
    assert_eq!(wide.cdf(-0.5 * f64::MAX), 0.25);
    assert_eq!(wide.cdf(-f64::MAX), 0.0);

    let shifted = UniformF64::new(-2.0, 2.0).unwrap();
    assert_eq!(shifted.density(), 0.25);
    assert_eq!(shifted.cdf(0.0), 0.5);
}

#[test]
fn increasing_input_produces_non_decreasing_samples() {
    // The mapping is monotone in the uniform input, so the test has to supply increasing inputs
    // rather than a random stream. Each step raises the top 53 bits by one, which is one step of the
    // `unit_f64` grid.
    let step = 1u64 << 11;
    let words: Vec<u64> = (0..1024).map(|index| index * step).collect();
    let mut source = SequenceQueue::of_u64(&words);
    let interval = UniformF64::new(-3.0, 7.0).unwrap();

    let mut previous = f64::NEG_INFINITY;
    for _ in 0..words.len() {
        let value = interval.sample(&mut source).unwrap();
        assert!(value >= previous, "the mapping is not monotone: {previous} then {value}");
        assert!((-3.0..7.0).contains(&value), "the mapping left the support with {value}");
        previous = value;
    }
}

#[test]
fn samples_are_bit_reproducible() {
    let bytes = {
        let mut stream = Stream::new(Seed::from_u64(0xf10a_7000), 1);
        let mut bytes = vec![0u8; 4096];
        stream.fill(&mut bytes).unwrap();
        bytes
    };
    let replay = |bytes: Vec<u8>| Replay { bytes, position: 0 };
    let interval = UniformF64::new(-1.0, 1.0).unwrap();
    let f32_interval = UniformF32::new(0.0, 1.0).unwrap();

    let mut first = replay(bytes.clone());
    let mut second = replay(bytes);
    for _ in 0..64 {
        assert_eq!(interval.sample(&mut first).unwrap().to_bits(), interval.sample(&mut second).unwrap().to_bits());
        assert_eq!(f32_interval.sample(&mut first).unwrap().to_bits(), f32_interval.sample(&mut second).unwrap().to_bits());
    }
}

/// Source that replays a fixed byte sequence.
struct Replay {
    bytes: Vec<u8>,
    position: usize,
}

impl RandomSource for Replay {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Error> {
        let end = self.position + dst.len();
        if end > self.bytes.len() {
            return Err(Error::new(ErrorKind::StreamExhausted));
        }
        dst.copy_from_slice(&self.bytes[self.position..end]);
        self.position = end;
        Ok(())
    }
}

#[test]
fn source_failures_are_reported() {
    let interval = UniformF64::new(0.0, 1.0).unwrap();
    assert_eq!(interval.sample(&mut FailingSource).unwrap_err().kind(), ErrorKind::BackendFailure);

    let interval = UniformF32::new(0.0, 1.0).unwrap();
    assert_eq!(interval.sample(&mut FailingSource).unwrap_err().kind(), ErrorKind::BackendFailure);
}

#[test]
#[ignore = "run manually to refresh the float sample goldens"]
fn record_float_sample_goldens() {
    let mut stream = Stream::new(Seed::from_u64(42), 0);
    let interval = UniformF64::new(-1.0, 1.0).unwrap();
    let bits: [u64; 4] = core::array::from_fn(|_| interval.sample(&mut stream).unwrap().to_bits());
    println!("f64 = {bits:#x?}");

    let mut stream = Stream::new(Seed::from_u64(42), 0);
    let interval = UniformF32::new(0.0, 1.0).unwrap();
    let bits: [u32; 4] = core::array::from_fn(|_| interval.sample(&mut stream).unwrap().to_bits());
    println!("f32 = {bits:#x?}");
}

#[test]
fn recorded_float_samples_are_locked() {
    let mut stream = Stream::new(Seed::from_u64(42), 0);
    let interval = UniformF64::new(-1.0, 1.0).unwrap();
    let expected: [u64; 4] = [0xbfc1_667d_af43_5390, 0xbfe5_b0e5_901b_8fc6, 0x3fd6_0936_0d5b_4db4, 0x3fd7_d0bd_1502_b37c];
    let actual: [u64; 4] = core::array::from_fn(|_| interval.sample(&mut stream).unwrap().to_bits());
    assert_eq!(actual, expected);

    let mut stream = Stream::new(Seed::from_u64(42), 0);
    let interval = UniformF32::new(0.0, 1.0).unwrap();
    let expected: [u32; 4] = [0x3f3c_ac71, 0x3edd_3304, 0x3f11_c0e9, 0x3e24_f1a4];
    let actual: [u32; 4] = core::array::from_fn(|_| interval.sample(&mut stream).unwrap().to_bits());
    assert_eq!(actual, expected);
}
