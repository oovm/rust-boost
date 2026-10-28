//! Structured diagnostic data model.

mod action;
mod code;
mod label;
mod location;
mod message;
mod origin;
mod record;
mod severity;

pub use crate::provenance::{AddressSpaceRef, DocumentRef, MappingPrecision, MemberPath, MemberSegment, ObjectRef, SemanticPath};
pub use action::{DiagnosticAction, DiagnosticCause};
pub use boost_source::{ByteRange, RangeError, SourceRef};
pub use code::DiagnosticCode;
pub use label::{DiagnosticLabel, LabelRole};
pub use location::DiagnosticLocation;
pub use message::{Message, MessageArg};
pub use origin::DiagnosticOrigin;
pub use record::Diagnostic;
pub use severity::DiagnosticSeverity;
