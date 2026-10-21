//! Compile-time check that facade-only consumers can use `#[boost::test::test]`.

use boost::test::assert_eq;

#[boost::test::test]
fn facade_smoke() {
    assert_eq!(1 + 1, 2);
}
