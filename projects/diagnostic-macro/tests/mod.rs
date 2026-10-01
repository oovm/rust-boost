/// Verify the placeholder proc macro expands in tests.
#[test]
fn real_macro_expands() {
    diagnostic_macro::real_macro!("11");
}
