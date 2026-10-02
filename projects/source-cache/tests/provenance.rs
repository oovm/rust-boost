use source_cache::{
    AddressSpaceRef, ByteRange, DocumentRef, MappingPrecision, MemberPath, MemberSegment, ObjectRef,
    SemanticPath, SourceRef,
};

#[test]
fn byte_range_rejects_inverted_bounds() {
    assert_eq!(ByteRange::new(4, 9).unwrap().is_empty(), false);
    assert!(ByteRange::new(9, 4).is_err());
}

#[test]
fn source_ref_wire_id_includes_revision() {
    let source = SourceRef::new("oak", "sample.xml").with_revision("1");
    assert_eq!(source.to_wire_id(), "oak:sample.xml@1");
}

#[test]
fn member_path_preserves_segments() {
    let path = MemberPath::new(vec![
        MemberSegment::new("zip", "word/document.xml"),
        MemberSegment::new("part", "document.xml"),
    ]);
    assert_eq!(path.segments().len(), 2);
    assert_eq!(path.segments()[0].kind(), "zip");
}

#[test]
fn object_and_document_refs_expose_kind_and_namespace() {
    let object = ObjectRef::new("pdf", "12 0");
    assert_eq!(object.kind(), "pdf");
    assert_eq!(object.id(), "12 0");

    let document = DocumentRef::new("notedown", "notes/main");
    assert_eq!(document.namespace(), "notedown");
}

#[test]
fn mapping_precision_serializes_kebab_case() {
    let value = serde_json::to_value(MappingPrecision::Container).unwrap();
    assert_eq!(value, "container");
}

#[test]
fn semantic_path_round_trips() {
    let path = SemanticPath::new("/blocks/17");
    assert_eq!(path.as_str(), "/blocks/17");
}

#[test]
fn address_space_ref_exposes_namespace() {
    let space = AddressSpaceRef::new("acorn", "decoded");
    assert_eq!(space.namespace(), "acorn");
    assert_eq!(space.id(), "decoded");
}
