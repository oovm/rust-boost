use source_cache::provenance::{
    AddressSpaceRef, ByteRange, DocumentRef, MappingPrecision, MemberPath, ObjectRef, SemanticPath,
    SourceRef,
};

/// Tagged diagnostic location across text, binary, container, object, and semantic domains.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", rename_all = "kebab-case"))]
pub enum DiagnosticLocation {
    /// UTF-8 text byte range in a source.
    Text {
        /// Source reference.
        source: SourceRef,
        /// Byte range in the UTF-8 view.
        range: ByteRange,
    },
    /// Binary byte range in an address space.
    Binary {
        /// Source reference.
        source: SourceRef,
        /// Address space reference.
        address_space: AddressSpaceRef,
        /// Byte range in the address space.
        range: ByteRange,
    },
    /// Container member with optional byte range.
    Member {
        /// Container source reference.
        container: SourceRef,
        /// Member path inside the container.
        member: MemberPath,
        /// Optional byte range within the member view.
        range: Option<ByteRange>,
        /// Mapping precision for provenance.
        precision: MappingPrecision,
    },
    /// Structured object reference.
    Object {
        /// Source reference.
        source: SourceRef,
        /// Object reference inside the source.
        object: ObjectRef,
    },
    /// Semantic path inside a document.
    Semantic {
        /// Document reference.
        document: DocumentRef,
        /// Semantic path inside the document.
        path: SemanticPath,
    },
    /// Virtual address space range layered on a source.
    Virtual {
        /// Source reference.
        source: SourceRef,
        /// Byte range in the virtual address space.
        range: ByteRange,
    },
}

impl DiagnosticLocation {
    /// Returns the wire kind string for this location variant.
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Text { .. } => "text",
            Self::Binary { .. } => "binary",
            Self::Member { .. } => "member",
            Self::Object { .. } => "object",
            Self::Semantic { .. } => "semantic",
            Self::Virtual { .. } => "virtual",
        }
    }
}
