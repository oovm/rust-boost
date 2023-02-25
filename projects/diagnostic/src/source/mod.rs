//! Opaque source references used by diagnostic locations.

pub use crate::model::{
    AddressSpaceRef, DocumentRef, MemberPath, MemberSegment, ObjectRef, SemanticPath, SourceRef,
};

/// Resolve opaque source references for renderers and providers.
///
/// Hosts implement this trait outside the core crate. It intentionally does not
/// own source storage or perform I/O by itself.
pub trait SourceLookup {
    /// Returns a stable wire identifier when the source is known.
    fn wire_id(&self, source: &SourceRef) -> Option<alloc::string::String>;
}
