use std::fmt::{self, Display, Formatter};
use std::string::String;
use std::vec::Vec;

/// Typed field value attached to console events and diagnostic projections.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldValue {
    /// Boolean value.
    Bool(bool),
    /// Signed integer.
    I64(i64),
    /// Unsigned integer.
    U64(u64),
    /// Floating point value.
    F64(f64),
    /// UTF-8 string.
    Str(String),
    /// Debug-formatted value for ad hoc logging.
    Debug(String),
}

impl FieldValue {
    /// Create a string field value.
    pub fn str(value: impl Into<String>) -> Self {
        Self::Str(value.into())
    }

    /// Create a debug-formatted field value.
    pub fn debug(value: impl fmt::Debug) -> Self {
        Self::Debug(format!("{value:?}"))
    }
}

impl Display for FieldValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(value) => write!(f, "{value}"),
            Self::I64(value) => write!(f, "{value}"),
            Self::U64(value) => write!(f, "{value}"),
            Self::F64(value) => write!(f, "{value}"),
            Self::Str(value) => f.write_str(value),
            Self::Debug(value) => f.write_str(value),
        }
    }
}

/// Named field on a console event.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    name: String,
    value: FieldValue,
}

impl Field {
    /// Create a named field.
    pub fn new(name: impl Into<String>, value: FieldValue) -> Self {
        Self { name: name.into(), value }
    }

    /// Returns the field name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the field value.
    pub fn value(&self) -> &FieldValue {
        &self.value
    }
}

/// Helper for building field lists.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fields(Vec<Field>);

impl Fields {
    /// Create an empty field list.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Append a field.
    pub fn push(&mut self, name: impl Into<String>, value: FieldValue) {
        self.0.push(Field::new(name, value));
    }

    /// Append a field and return `self`.
    pub fn field(mut self, name: impl Into<String>, value: FieldValue) -> Self {
        self.push(name, value);
        self
    }

    /// Returns the underlying fields.
    pub fn as_slice(&self) -> &[Field] {
        &self.0
    }

    /// Consumes the builder and returns the fields.
    pub fn into_vec(self) -> Vec<Field> {
        self.0
    }
}
