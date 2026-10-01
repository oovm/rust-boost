use crate::QError;
pub use rust_decimal::Decimal;
use rust_decimal::Error;

impl From<Error> for QError {
    fn from(error: Error) -> Self {
        QError::wrap_syntax_error(error)
    }
}
