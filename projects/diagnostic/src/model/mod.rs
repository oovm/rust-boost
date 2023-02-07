//! Structured diagnostic data model.

mod code;
mod label;
mod location;
mod message;
mod origin;
mod record;
mod recovery;
mod severity;

pub use code::DiagnosticCode;
pub use label::{DiagnosticLabel, LabelRole};
pub use location::{
    AddressSpaceRef, ByteRange, DiagnosticLocation, DocumentRef, MappingPrecision, MemberPath, MemberSegment, ObjectRef,
    SemanticPath, SourceRef,
};
pub use message::{Message, MessageArg};
pub use origin::DiagnosticOrigin;
pub use record::Diagnostic;
pub use recovery::{DiagnosticCause, RecoveryAction};
pub use severity::DiagnosticSeverity;
