mod expand_usage;

#[test]
fn integration_smoke() {
    expand_usage::sample().unwrap();
}
