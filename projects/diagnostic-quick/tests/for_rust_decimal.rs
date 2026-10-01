use diagnostic_quick::QResult;
use rust_decimal::Decimal;
use std::str::FromStr;

#[test]
fn rust_decimal_from_str() -> QResult {
    assert!(Decimal::from_str("0").is_ok());
    assert!(Decimal::from_scientific("1e+10").is_ok());
    assert!(Decimal::from_scientific("1e-10").is_ok());
    Ok(())
}
