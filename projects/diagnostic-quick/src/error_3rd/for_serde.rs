use std::fmt::Display;

use serde::{
    de::{Expected, Unexpected},
    ser::Error,
};

use diagnostic::DiagnosticSeverity;

use crate::{QError, QErrorKind};

impl Error for QError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        QError {
            error: Box::new(QErrorKind::Custom(msg.to_string())),
            severity: DiagnosticSeverity::Error,
            source: None,
        }
    }
}

#[allow(unused_variables)]
impl serde::de::Error for QError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::custom(msg)
    }

    fn invalid_type(unexp: Unexpected, exp: &dyn Expected) -> Self {
        todo!()
    }

    fn invalid_value(unexp: Unexpected, exp: &dyn Expected) -> Self {
        todo!()
    }

    fn invalid_length(len: usize, exp: &dyn Expected) -> Self {
        todo!()
    }

    fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
        todo!()
    }

    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
        todo!()
    }

    fn missing_field(field: &'static str) -> Self {
        todo!()
    }

    fn duplicate_field(field: &'static str) -> Self {
        todo!()
    }
}
