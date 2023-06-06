use std::borrow::Cow;

use boost_log::{FieldValue, Level, LogEvent, event_to_json, field_value};

#[test]
fn signed_integers_stay_signed() {
    assert_eq!(field_value!(-7i8), FieldValue::I64(-7));
    assert_eq!(field_value!(-300i32), FieldValue::I64(-300));
    assert_eq!(field_value!(i64::MIN), FieldValue::I64(i64::MIN));
    assert_eq!(field_value!(-2isize), FieldValue::I64(-2));
}

#[test]
fn unsigned_integers_stay_unsigned() {
    assert_eq!(field_value!(7u8), FieldValue::U64(7));
    assert_eq!(field_value!(300u32), FieldValue::U64(300));
    assert_eq!(field_value!(u64::MAX), FieldValue::U64(u64::MAX));
    assert_eq!(field_value!(2usize), FieldValue::U64(2));
}

#[test]
fn floats_booleans_and_characters_keep_their_type() {
    assert_eq!(field_value!(0.5f32), FieldValue::F64(0.5));
    assert_eq!(field_value!(1.25f64), FieldValue::F64(1.25));
    assert_eq!(field_value!(true), FieldValue::Bool(true));
    assert_eq!(field_value!('v'), FieldValue::Str("v".to_string()));
}

#[test]
fn strings_keep_their_type_whatever_their_owner_is() {
    assert_eq!(field_value!("literal"), FieldValue::Str("literal".to_string()));

    let owned = String::from("owned");
    assert_eq!(field_value!(owned.as_str()), FieldValue::Str("owned".to_string()));
    assert_eq!(field_value!(&owned), FieldValue::Str("owned".to_string()));
    assert_eq!(field_value!(owned.clone()), FieldValue::Str("owned".to_string()));
    assert_eq!(field_value!(Cow::Borrowed("cow")), FieldValue::Str("cow".to_string()));
}

#[test]
fn an_explicit_field_value_is_passed_through() {
    let explicit = FieldValue::debug(Some(3));
    assert_eq!(field_value!(explicit.clone()), explicit);
}

#[test]
fn json_writes_typed_fields_without_quoting_them() {
    let event = LogEvent::log(Level::Info, "worker")
        .field("attempt", field_value!(3u32))
        .field("delta", field_value!(-2i64))
        .field("ratio", field_value!(0.5f64))
        .field("ready", field_value!(true))
        .field("message", field_value!("done"))
        .build();

    let json = event_to_json(&event);

    assert!(json.contains("\"attempt\":3"), "{json}");
    assert!(json.contains("\"delta\":-2"), "{json}");
    assert!(json.contains("\"ratio\":0.5"), "{json}");
    assert!(json.contains("\"ready\":true"), "{json}");
    assert!(json.contains("\"message\":\"done\""), "{json}");
}

#[test]
fn a_debug_field_stays_a_quoted_string() {
    let event = LogEvent::log(Level::Info, "worker").field("path", FieldValue::debug(Some("x"))).build();

    let json = event_to_json(&event);

    assert!(json.contains("\"path\":\"Some(\\\"x\\\")\""), "{json}");
}
