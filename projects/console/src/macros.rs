/// Emit a structured console event.
#[macro_export]
macro_rules! event {
    ($level:ident, $target:expr) => {
        $crate::emit($crate::ConsoleEvent::log($crate::Level::$level, $target).build())
    };
    ($level:ident, $target:expr, $($key:ident = $value:expr),+ $(,)?) => {
        $crate::emit({
            let mut builder = $crate::ConsoleEvent::log($crate::Level::$level, $target);
            $(builder = builder.field(stringify!($key), $crate::field_value!($value));)*
            builder.build()
        })
    };
}

/// Create a console span.
#[macro_export]
macro_rules! span {
    ($target:expr) => {
        $crate::ConsoleSpan::new($target)
    };
    ($target:expr, $($key:ident = $value:expr),+ $(,)?) => {
        {
            let mut span = $crate::ConsoleSpan::new($target);
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

macro_rules! style_builder_for {
    ($T:ty, |$s:ident| $props:expr, $($name:ident: $property:ident),*) => ($(
        #[doc = concat!(
            "Enables the _", stringify!($name), "_ style on `self`.\n",
            "```rust\n",
            "use console::Paint;\n",
            "\n",
            "println!(\"Using ", stringify!($name), ": {}\", ",
                "Paint::new(\"hi\").", stringify!($name), "());\n",
            "```\n"
        )]
        #[inline]
        pub fn $name(self) -> $T {
            let mut $s = self;
            $props.set(Property::$property);
            $s
        }
    )*)
}
