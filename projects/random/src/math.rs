//! The arithmetic kernel used by the continuous distributions.
//!
//! The crate is `no_std` and does not link a math library, so the two functions that the continuous
//! distributions need are implemented here. Owning them keeps a distributed build from picking up a
//! platform `libm` whose last bits differ per target.
//!
//! Owning the kernel does not by itself make the output portable. `Ln2` and the `inverse_hyperbolic`
//! series are evaluated in IEEE arithmetic, so a target that rounds each operation the same way
//! produces the same bits, but the accuracy contract is stated as an error bound rather than a
//! bit-for-bit promise, and the `normal_exponential` tests measure it against the host standard
//! library through the public sampling entries.

/// Natural logarithm of two, split so that reducing a large argument does not lose its own last
/// bits: a single rounded constant would turn into a relative error in the exponential.
const LN_2_HI: f64 = core::f64::consts::LN_2;

/// The part of `ln 2` that does not fit in [`LN_2_HI`].
const LN_2_LO: f64 = 2.319_046_813_846_299_6e-17;

/// Reads the exponent field of an `f64`.
fn exponent_of(value: f64) -> i32 {
    ((value.to_bits() >> 52) & 0x7ff) as i32
}

/// Replaces the exponent field of an `f64`, keeping its sign and mantissa.
fn with_exponent(value: f64, exponent: i32) -> f64 {
    let sign = value.to_bits() & 0x8000_0000_0000_0000;
    f64::from_bits(sign | ((exponent as u64) << 52) | (value.to_bits() & 0x000f_ffff_ffff_ffff))
}

/// Returns the square root of `value`.
///
/// A negative input returns `NaN`, and the zero, infinity, and `NaN` inputs return themselves, so
/// this matches the domain of the standard function. Subnormal inputs are scaled into the normal
/// range before the iteration and scaled back afterwards, which avoids the loss of precision that a
/// direct computation on a subnormal would cause.
pub(crate) fn sqrt(value: f64) -> f64 {
    if value.is_nan() || value < 0.0 || value == f64::INFINITY || value == 0.0 {
        return if value < 0.0 { f64::NAN } else { value };
    }

    // A subnormal has no usable exponent, so it is scaled up and the result is scaled back.
    if exponent_of(value) == 0 {
        const SCALE: f64 = 18_014_398_509_481_984.0; // 2^54
        return sqrt(value * SCALE) / 9_007_199_254_740_992.0; // 2^27
    }

    // The classic bit-level initial guess halves the exponent, then Newton iteration refines it.
    // Each step roughly doubles the number of correct digits, so a handful reach the last bit.
    let mut guess = f64::from_bits(0x1ff7_ffff_ffff_ffff + (value.to_bits() >> 1));
    for _ in 0..6 {
        guess = 0.5 * (guess + value / guess);
    }
    guess
}

/// Returns the natural logarithm of `value`.
///
/// A zero input returns negative infinity, a negative input returns `NaN`, and the infinity and
/// `NaN` inputs return themselves. The value is split into `m * 2^k` with `m` inside
/// `[sqrt(1/2), sqrt(2))`, so the series below converges quickly and the exponent contributes the
/// exact multiple of `Ln2`. Subnormal inputs are scaled into the normal range first.
pub(crate) fn ln(value: f64) -> f64 {
    if value.is_nan() || value == f64::INFINITY || value < 0.0 {
        return if value < 0.0 { f64::NAN } else { value };
    }
    if value == 0.0 {
        return f64::NEG_INFINITY;
    }

    let mut scaled = value;
    let mut exponent = 0i32;
    if exponent_of(scaled) == 0 {
        const SCALE: f64 = 18_014_398_509_481_984.0; // 2^54
        scaled *= SCALE;
        exponent = -54;
    }

    let exponent_field = exponent_of(scaled);
    exponent += exponent_field - 1023;
    // Normalizing the mantissa to `[1, 2)` and then to `[sqrt(1/2), sqrt(2))` keeps `t` small.
    let mut mantissa = with_exponent(scaled, 1023);
    if mantissa > SQRT_2 {
        mantissa *= 0.5;
        exponent += 1;
    }

    let t = (mantissa - 1.0) / (mantissa + 1.0);
    let squared = t * t;
    // `ln(m) = 2 * atanh(t) = 2 * t * (1 + t^2/3 + t^4/5 + ...)`, and `|t| < 0.18` here, so the
    // remaining terms are below the last bit of the result.
    let mut term = 1.0;
    let mut sum = 0.0;
    let mut denominator = 1.0;
    for _ in 0..10 {
        sum += term / denominator;
        term *= squared;
        denominator += 2.0;
    }
    let adjustment = 2.0 * t * sum;

    (exponent as f64) * LN_2_HI + (exponent as f64) * LN_2_LO + adjustment
}

/// Square root of two, the threshold that keeps the logarithm series argument small.
const SQRT_2: f64 = core::f64::consts::SQRT_2;

/// Reciprocal of `ln 2`, used to split an exponential argument into a power of two and a remainder.
const INVERSE_LN_2: f64 = core::f64::consts::LOG2_E;

/// Largest argument whose exponential is still representable.
const EXP_OVERFLOW: f64 = 709.782_712_893_384;

/// Smallest argument whose exponential is still above zero after rounding.
const EXP_UNDERFLOW: f64 = -745.133_219_101_941_1;

/// Returns `e` raised to `value`.
///
/// The argument is split into `k * ln 2` and a remainder inside `[-ln 2 / 2, ln 2 / 2]`, so the
/// Taylor series below converges in a few terms and the power of two is applied exactly at the end.
/// Overflow returns infinity and underflow returns zero, matching the format's own limits.
pub(crate) fn exp(value: f64) -> f64 {
    if value.is_nan() || value == f64::INFINITY {
        return value;
    }
    if value == f64::NEG_INFINITY {
        return 0.0;
    }
    if value > EXP_OVERFLOW {
        return f64::INFINITY;
    }
    if value < EXP_UNDERFLOW {
        return 0.0;
    }

    // Rounding to the nearest multiple of `ln 2` keeps the remainder small on both sides of zero.
    let steps = if value >= 0.0 { (value * INVERSE_LN_2 + 0.5) as i32 } else { (value * INVERSE_LN_2 - 0.5) as i32 };
    // Reducing by a multiple of `ln 2` cancels two nearly equal values, so the product is kept exact:
    // the residual that the rounded product dropped is recovered below, and the remainder then carries
    // the error of the last bit instead of the error of the whole product. `f64::mul_add` would fuse
    // this, but it lives in the standard library, which a `no_std` crate cannot use.
    let steps_as_float = f64::from(steps);
    let product = steps_as_float * LN_2_HI;
    let product_error = product_error(steps_as_float, LN_2_HI, product);
    let remainder = (value - product) - (product_error + steps_as_float * LN_2_LO);

    let mut term = 1.0;
    let mut sum = 1.0;
    for index in 1..=16 {
        term *= remainder / f64::from(index);
        sum += term;
    }
    sum * power_of_two(steps)
}

/// Returns `2` raised to an integer power, using the exact encoding of the format.
fn power_of_two(exponent: i32) -> f64 {
    if exponent >= -1022 {
        f64::from_bits(((exponent + 1023) as u64) << 52)
    }
    else if exponent >= -1074 {
        f64::from_bits(1u64 << (exponent + 1074))
    }
    else {
        0.0
    }
}

/// Splits `value` into two parts whose sum is exact and whose significands each hold at most 26 bits.
///
/// The splitter is `2^27 + 1`, the classic choice for `f64`, and it requires that the intermediate
/// product stays inside the format, which holds for every value this module splits.
fn split(value: f64) -> (f64, f64) {
    const SPLITTER: f64 = 134_217_729.0;
    let scaled = SPLITTER * value;
    let high = scaled - (scaled - value);
    (high, value - high)
}

/// Returns the residual that the rounded `product` of `left * right` dropped.
///
/// Dekker's algorithm recovers it from the split halves instead of a fused multiply-add, so the
/// reduction of an exponential argument keeps its last bits without the standard library.
fn product_error(left: f64, right: f64, product: f64) -> f64 {
    let (left_high, left_low) = split(left);
    let (right_high, right_low) = split(right);
    ((left_high * right_high - product) + left_high * right_low + left_low * right_high) + left_low * right_low
}
