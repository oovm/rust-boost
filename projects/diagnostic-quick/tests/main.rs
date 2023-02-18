mod convert;

#[cfg(feature = "rust_decimal")]
mod for_rust_decimal;

use diagnostic_quick::QResult;

#[test]
fn ready() -> QResult {
    let _: diagnostic_quick::QError = std::io::Error::new(std::io::ErrorKind::Other, "io").into();
    Ok(())
}
