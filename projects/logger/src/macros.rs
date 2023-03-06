/// Emit a structured log event.
#[macro_export]
macro_rules! event {
    ($level:ident, $target:expr) => {
        $crate::emit($crate::LogEvent::log($crate::Level::$level, $target).build())
    };
    ($level:ident, $target:expr, $($key:ident = $value:expr),+ $(,)?) => {
        $crate::emit({
            let mut builder = $crate::LogEvent::log($crate::Level::$level, $target);
            $(builder = builder.field(stringify!($key), $crate::field_value!($value));)*
            builder.build()
        })
    };
}

/// Create a log span.
#[macro_export]
macro_rules! span {
    ($target:expr) => {
        $crate::LogSpan::new($target)
    };
    ($target:expr, $($key:ident = $value:expr),+ $(,)?) => {
        {
            let mut span = $crate::LogSpan::new($target);
            $(span = span.field(stringify!($key), $crate::field_value!($value));)*
            span
        }
    };
}

/// Convert supported literal types into [`FieldValue`].
#[macro_export]
macro_rules! field_value {
    ($value:expr) => {
        match &$value {
            value => $crate::FieldValue::debug(value),
        }
    };
}
