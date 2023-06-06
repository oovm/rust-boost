//! Stable text projection of structured test results.
//!
//! The projection only renders facts that the runner already recorded. It does not recompute the
//! outcome, reorder failures, or turn a skipped entry into a passing one.

use core::fmt::Write;

use crate::result::{TestOutcome, TestResult};

/// Renders the stable text projection of a test result.
///
/// The first line describes the entry and its outcome. Each failure then contributes a phase/kind
/// line, a message, and its operands in declaration order.
pub fn render_text(result: &TestResult) -> String {
    let mut text = String::new();
    let name = result.descriptor().qualified_name();
    let millis = result.duration().as_secs_f64() * 1_000.0;
    match result.outcome() {
        TestOutcome::Passed => {
            let _ = writeln!(text, "test {name} ... ok ({millis:.3} ms)");
        }
        TestOutcome::ExpectedFailure => {
            let _ = writeln!(text, "test {name} ... expected failure ({millis:.3} ms)");
        }
        TestOutcome::Skipped => match result.skip_reason() {
            Some(reason) => {
                let _ = writeln!(text, "test {name} ... skipped: {reason} ({millis:.3} ms)");
            }
            None => {
                let _ = writeln!(text, "test {name} ... skipped ({millis:.3} ms)");
            }
        },
        TestOutcome::Failed => {
            let _ = writeln!(text, "test {name} ... FAILED ({millis:.3} ms)");
            for failure in result.failures() {
                match failure.location() {
                    Some(location) => {
                        let _ = writeln!(text, "  [{} {}] at {location}", failure.phase(), failure.kind());
                    }
                    None => {
                        let _ = writeln!(text, "  [{} {}]", failure.phase(), failure.kind());
                    }
                }
                let _ = writeln!(text, "    {}", failure.message());
                for operand in failure.operands() {
                    let _ = writeln!(text, "    {operand}");
                }
            }
        }
    }
    text
}
