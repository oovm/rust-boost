use std::env;

use logger::{Filter, FilterParseError, Level, LogEvent};
use serial_test::serial;

fn event(level: Level, target: &str) -> LogEvent {
    LogEvent::log(level, target).build()
}

#[test]
fn a_global_level_applies_to_every_target() {
    let filter = Filter::parse("info").expect("filter parses");

    assert_eq!(filter.min_level(), Some(Level::Info));
    assert!(filter.accepts(&event(Level::Info, "oak")));
    assert!(filter.accepts(&event(Level::Error, "vmz")));
    assert!(!filter.accepts(&event(Level::Debug, "oak::syntax")));
}

#[test]
fn levels_are_parsed_without_regard_to_case() {
    assert_eq!(Filter::parse("WARN").expect("filter parses").min_level(), Some(Level::Warn));
    assert_eq!(Filter::parse(" Error ").expect("filter parses").min_level(), Some(Level::Error));
}

#[test]
fn a_target_directive_drops_targets_it_does_not_name() {
    let filter = Filter::parse("oak=debug").expect("filter parses");

    assert!(filter.accepts(&event(Level::Debug, "oak::syntax")));
    assert!(filter.accepts(&event(Level::Error, "oak")));
    assert!(!filter.accepts(&event(Level::Trace, "oak::syntax")));
    assert!(!filter.accepts(&event(Level::Error, "vmz")));
    assert_eq!(filter.level_for("vmz"), None);
}

#[test]
fn the_longest_matching_target_wins() {
    let filter = Filter::parse("info,oak=warn,oak::syntax=debug").expect("filter parses");

    assert!(filter.accepts(&event(Level::Info, "vmz")));
    assert!(!filter.accepts(&event(Level::Debug, "vmz")));
    assert!(!filter.accepts(&event(Level::Info, "oak")));
    assert!(filter.accepts(&event(Level::Warn, "oak::parser")));
    assert!(filter.accepts(&event(Level::Debug, "oak::syntax")));
    assert!(!filter.accepts(&event(Level::Trace, "oak::syntax")));
}

#[test]
fn a_target_directive_stops_at_a_path_boundary() {
    let filter = Filter::parse("oak=debug").expect("filter parses");

    assert!(filter.accepts(&event(Level::Debug, "oak")));
    assert!(!filter.accepts(&event(Level::Error, "oak_vfs")));
}

#[test]
fn a_bare_target_accepts_every_level() {
    let filter = Filter::parse("oak,vmz=warn").expect("filter parses");

    assert!(filter.accepts(&event(Level::Trace, "oak::syntax")));
    assert!(!filter.accepts(&event(Level::Info, "vmz")));
    assert!(!filter.accepts(&event(Level::Error, "nifty")));
}

#[test]
fn an_empty_description_is_rejected() {
    assert_eq!(Filter::parse("   "), Err(FilterParseError::Empty));
}

#[test]
fn an_empty_directive_is_rejected() {
    assert_eq!(Filter::parse("oak=info,"), Err(FilterParseError::EmptyDirective { position: 2 }));
}

#[test]
fn an_empty_target_is_rejected() {
    assert_eq!(Filter::parse("=info"), Err(FilterParseError::EmptyTarget { directive: "=info".to_string() }));
}

#[test]
fn an_empty_level_is_rejected() {
    assert_eq!(Filter::parse("oak="), Err(FilterParseError::EmptyLevel { directive: "oak=".to_string() }));
}

#[test]
fn an_unknown_level_is_rejected() {
    assert_eq!(
        Filter::parse("oak=verbose"),
        Err(FilterParseError::UnknownLevel { directive: "oak=verbose".to_string(), level: "verbose".to_string() })
    );
}

#[test]
fn a_repeated_target_is_rejected() {
    assert_eq!(Filter::parse("oak=info,oak=warn"), Err(FilterParseError::DuplicateTarget { target: "oak".to_string() }));
}

#[test]
fn a_repeated_global_level_is_rejected() {
    assert_eq!(Filter::parse("info,warn"), Err(FilterParseError::DuplicateGlobalLevel { directive: "warn".to_string() }));
}

#[test]
fn a_filter_built_in_code_keeps_the_global_level() {
    let filter = Filter::new(Level::Warn).with_target("oak", Level::Trace);

    assert_eq!(filter.min_level(), Some(Level::Warn));
    assert!(filter.accepts(&event(Level::Trace, "oak::syntax")));
    assert!(filter.accepts(&event(Level::Warn, "vmz")));
    assert!(!filter.accepts(&event(Level::Info, "vmz")));
}

#[test]
#[serial]
fn a_missing_environment_variable_is_not_a_filter() {
    let name = "RUST_BOOST_LOGGER_FILTER_ABSENT";
    // The test is serialized, so no other test reads this variable concurrently.
    unsafe { env::remove_var(name) };

    assert_eq!(Filter::from_env(name), Ok(None));
}

#[test]
#[serial]
fn an_environment_filter_is_parsed() {
    let name = "RUST_BOOST_LOGGER_FILTER_VALID";
    // The test is serialized, so no other test reads this variable concurrently.
    unsafe { env::set_var(name, "oak=debug") };

    let filter = Filter::from_env(name).expect("filter parses").expect("variable is set");

    assert!(filter.accepts(&event(Level::Debug, "oak::syntax")));
    assert!(!filter.accepts(&event(Level::Error, "vmz")));

    unsafe { env::remove_var(name) };
}

#[test]
#[serial]
fn an_invalid_environment_filter_reports_the_directive() {
    let name = "RUST_BOOST_LOGGER_FILTER_INVALID";
    // The test is serialized, so no other test reads this variable concurrently.
    unsafe { env::set_var(name, "oak=verbose") };

    assert_eq!(
        Filter::from_env(name),
        Err(FilterParseError::UnknownLevel { directive: "oak=verbose".to_string(), level: "verbose".to_string() })
    );

    unsafe { env::remove_var(name) };
}
