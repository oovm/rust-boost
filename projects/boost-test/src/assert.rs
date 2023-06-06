//! Structured assertions for test bodies.
//!
//! An assertion fails by panicking with a [`TestFailure`] payload. [`execute`](crate::execute)
//! catches that payload and keeps its source location and operands, so the body still stops exactly
//! where the assertion failed. A panic payload is used because a body that returns `()` has no other
//! channel for an immediate abort; the context is not hidden in a thread-local.

use core::fmt::Debug;

use crate::{
    descriptor::SourceLocation,
    result::{AssertionKind, Operand, TestFailure},
};

/// Builds the source location used by an assertion macro.
///
/// This is public because macro expansion runs in the caller crate; it is not part of the intended
/// author surface.
#[doc(hidden)]
pub fn location(file: &'static str, line: u32, column: u32) -> SourceLocation {
    SourceLocation::new(file, line, column)
}

/// Raises a structured failure as the panic payload of the current body.
///
/// The runner downcasts the payload back to [`TestFailure`]. Calling this outside a `testkit`
/// runner stops the current thread like any other panic.
pub fn raise(failure: TestFailure) -> ! {
    std::panic::panic_any(failure)
}

/// Fails when `condition` is false.
///
/// The macro [`assert!`](crate::assert!) calls this function with the caller location.
pub fn condition(condition: bool, message: Option<String>, location: SourceLocation) {
    if !condition {
        let message = message.unwrap_or_else(|| "assertion failed".to_owned());
        raise(TestFailure::assertion(AssertionKind::Condition, message, location, Vec::new()));
    }
}

/// Fails when `left` and `right` are equal.
///
/// The macro [`assert_eq!`](crate::assert_eq!) calls this function with the caller location.
pub fn equal<A, B>(left: &A, right: &B, message: Option<String>, location: SourceLocation)
where
    A: PartialEq<B> + Debug,
    B: Debug,
{
    if left == right {
        return;
    }
    let message = message.unwrap_or_else(|| "assertion failed: left == right".to_owned());
    let operands = vec![Operand::debug("left", left), Operand::debug("right", right)];
    raise(TestFailure::assertion(AssertionKind::Equal, message, location, operands));
}

/// Fails when `left` and `right` are different.
///
/// The macro [`assert_ne!`](crate::assert_ne!) calls this function with the caller location.
pub fn not_equal<A, B>(left: &A, right: &B, message: Option<String>, location: SourceLocation)
where
    A: PartialEq<B> + Debug,
    B: Debug,
{
    if left != right {
        return;
    }
    let message = message.unwrap_or_else(|| "assertion failed: left != right".to_owned());
    let operands = vec![Operand::debug("left", left), Operand::debug("right", right)];
    raise(TestFailure::assertion(AssertionKind::NotEqual, message, location, operands));
}

/// Raises an explicit failure.
///
/// The macro [`fail!`](crate::fail!) calls this function with the caller location.
pub fn failure(message: impl Into<String>, location: SourceLocation) {
    raise(TestFailure::assertion(AssertionKind::Fail, message, location, Vec::new()));
}

/// Fails the current test when `condition` is false.
///
/// ```ignore
/// boost_test::assert!(value > 0);
/// boost_test::assert!(value > 0, "value must be positive, got {value}");
/// ```
#[macro_export]
macro_rules! assert {
    ($condition:expr $(,)?) => {{
        $crate::assert::condition($condition, ::core::option::Option::None, $crate::assert::location(file!(), line!(), column!()));
    }};
    ($condition:expr, $($message:tt)+) => {{
        $crate::assert::condition(
            $condition,
            ::core::option::Option::Some(::std::format!($($message)+)),
            $crate::assert::location(file!(), line!(), column!()),
        );
    }};
}

/// Fails the current test when the two values are equal.
///
/// ```ignore
/// boost_test::assert_eq!(left, right);
/// boost_test::assert_eq!(left, right, "left and right diverged");
/// ```
#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr $(,)?) => {{
        $crate::assert::equal(
            &$left,
            &$right,
            ::core::option::Option::None,
            $crate::assert::location(file!(), line!(), column!()),
        );
    }};
    ($left:expr, $right:expr, $($message:tt)+) => {{
        $crate::assert::equal(
            &$left,
            &$right,
            ::core::option::Option::Some(::std::format!($($message)+)),
            $crate::assert::location(file!(), line!(), column!()),
        );
    }};
}

/// Fails the current test when the two values are different.
///
/// ```ignore
/// boost_test::assert_ne!(left, right);
/// boost_test::assert_ne!(left, right, "left and right must differ");
/// ```
#[macro_export]
macro_rules! assert_ne {
    ($left:expr, $right:expr $(,)?) => {{
        $crate::assert::not_equal(
            &$left,
            &$right,
            ::core::option::Option::None,
            $crate::assert::location(file!(), line!(), column!()),
        );
    }};
    ($left:expr, $right:expr, $($message:tt)+) => {{
        $crate::assert::not_equal(
            &$left,
            &$right,
            ::core::option::Option::Some(::std::format!($($message)+)),
            $crate::assert::location(file!(), line!(), column!()),
        );
    }};
}

/// Fails the current test with an explicit message.
///
/// ```ignore
/// boost_test::fail!("the invariant is broken");
/// boost_test::fail!("unexpected value: {}", value);
/// ```
#[macro_export]
macro_rules! fail {
    () => {{
        $crate::assert::failure("explicit failure", $crate::assert::location(file!(), line!(), column!()));
    }};
    ($($message:tt)+) => {{
        $crate::assert::failure(::std::format!($($message)+), $crate::assert::location(file!(), line!(), column!()));
    }};
}
