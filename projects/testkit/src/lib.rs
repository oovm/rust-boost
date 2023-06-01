#![doc = include_str!("readme.md")]
#![deny(missing_docs)]

pub mod assert;
pub mod case;
pub mod context;
pub mod descriptor;
pub mod fixture;
pub mod report;
pub mod result;
pub mod runner;

#[cfg(feature = "macros")]
pub use testkit_macro::test;

pub use crate::{
    case::{CaseError, CaseId},
    context::TestContext,
    descriptor::{Capability, Expectation, SourceLocation, TestDescriptor},
    fixture::{Fixture, FixtureDescriptor, IntoTeardown},
    report::render_text,
    result::{AssertionKind, FailureKind, IntoTestFailure, Operand, OperandValue, Phase, TestFailure, TestOutcome, TestResult},
    runner::{execute, execute_with_fixture, run, run_with_fixture},
};
