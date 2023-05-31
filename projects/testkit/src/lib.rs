#![doc = include_str!("readme.md")]
#![deny(missing_docs)]

pub mod assert;
pub mod case;
pub mod context;
pub mod descriptor;
pub mod report;
pub mod result;
pub mod runner;

pub use crate::{
    case::{CaseError, CaseId},
    context::TestContext,
    descriptor::{Capability, Expectation, SourceLocation, TestDescriptor},
    report::render_text,
    result::{AssertionKind, FailureKind, IntoTestFailure, Operand, OperandValue, Phase, TestFailure, TestOutcome, TestResult},
    runner::{execute, run},
};
