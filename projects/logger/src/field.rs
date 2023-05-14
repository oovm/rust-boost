use std::{
    fmt::{self, Display, Formatter},
    string::String,
    vec::Vec,
};

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

    /// Create a field value from a type that [`IntoFieldValue`] keeps intact.
    ///
    /// A value whose type is not understood has to go through [`FieldValue::debug`] instead of
    /// being silently stringified.
    pub fn from_value(value: impl IntoFieldValue) -> Self {
        value.into_field_value()
    }
}

/// Converts a value into a [`FieldValue`] without erasing its type.
///
/// Only types this crate can represent keep their meaning: booleans, integers, floats, strings,
/// characters, and [`FieldValue`] itself. Every other type has to be wrapped with
/// [`FieldValue::debug`], so an untyped value cannot quietly become a `Debug` string.
pub trait IntoFieldValue {
    /// Convert the value.
    fn into_field_value(self) -> FieldValue;
}

impl IntoFieldValue for FieldValue {
    fn into_field_value(self) -> FieldValue {
        self
    }
}

impl IntoFieldValue for bool {
    fn into_field_value(self) -> FieldValue {
        FieldValue::Bool(self)
    }
}

macro_rules! signed_field_values {
    ($($ty:ty => $convert:expr),+ $(,)?) => {
        $(impl IntoFieldValue for $ty {
            fn into_field_value(self) -> FieldValue {
                FieldValue::I64($convert(self))
            }
        })+
    };
}

signed_field_values!(i8 => i64::from, i16 => i64::from, i32 => i64::from, i64 => |value| value, isize => |value| value as i64,);

macro_rules! unsigned_field_values {
    ($($ty:ty => $convert:expr),+ $(,)?) => {
        $(impl IntoFieldValue for $ty {
            fn into_field_value(self) -> FieldValue {
                FieldValue::U64($convert(self))
            }
        })+
    };
}

unsigned_field_values!(u8 => u64::from, u16 => u64::from, u32 => u64::from, u64 => |value| value, usize => |value| value as u64,);

impl IntoFieldValue for f32 {
    fn into_field_value(self) -> FieldValue {
        FieldValue::F64(f64::from(self))
    }
}

impl IntoFieldValue for f64 {
    fn into_field_value(self) -> FieldValue {
        FieldValue::F64(self)
    }
}

impl IntoFieldValue for char {
    fn into_field_value(self) -> FieldValue {
        FieldValue::Str(self.to_string())
    }
}

impl IntoFieldValue for &str {
    fn into_field_value(self) -> FieldValue {
        FieldValue::Str(self.to_string())
    }
}

impl IntoFieldValue for String {
    fn into_field_value(self) -> FieldValue {
        FieldValue::Str(self)
    }
}

impl IntoFieldValue for &String {
    fn into_field_value(self) -> FieldValue {
        FieldValue::Str(self.clone())
    }
}

impl IntoFieldValue for std::borrow::Cow<'_, str> {
    fn into_field_value(self) -> FieldValue {
        FieldValue::Str(self.into_owned())
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
