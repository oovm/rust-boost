use source::{
    AccessError, ByteAccess, ByteRange, LineIndex, MemorySnapshot, MemoryStore, PartialByteAccess, SnapshotHandle,
    SnapshotRef, SourceRef, decode_source_ref, encode_source_ref,
};

#[test]
fn byte_range_rejects_inverted_bounds() {
    assert!(!ByteRange::new(4, 9).unwrap().is_empty());
    assert!(ByteRange::new(9, 4).is_err());
}

#[test]
fn byte_range_half_open_contains_start_not_end() {
    let range = ByteRange::new(2, 5).unwrap();
    assert!(range.contains(2));
    assert!(range.contains(4));
    assert!(!range.contains(5));
}

#[test]
fn empty_input_has_single_line_at_zero() {
    let lines = LineIndex::from_bytes(b"");
    assert_eq!(lines.line_count(), 1);
    assert_eq!(lines.line_offset(0), Some(0));
    assert_eq!(lines.coords_at_offset(0, 0).unwrap(), source::TextCoords { line: 0, column: 0 });
}

#[test]
fn trailing_whitespace_is_preserved() {
    let bytes = b"hello   \nworld";
    let snapshot = MemorySnapshot::from_bytes(
        SnapshotRef::new(SourceRef::new("oak", "sample.nd").unwrap()),
        bytes.to_vec(),
    );
    assert_eq!(snapshot.text_view().line_at(0).unwrap(), b"hello   ");
}

#[test]
fn crlf_and_lone_cr_split_lines() {
    let bytes = b"a\r\nb\rc\n";
    let lines = LineIndex::from_bytes(bytes);
    assert_eq!(lines.line_count(), 3);
    let snapshot = MemorySnapshot::from_bytes(
        SnapshotRef::new(SourceRef::new("oak", "sample.nd").unwrap()),
        bytes.to_vec(),
    );
    assert_eq!(snapshot.text_view().line_at(0).unwrap(), b"a");
    assert_eq!(snapshot.text_view().line_at(1).unwrap(), b"b");
    assert_eq!(snapshot.text_view().line_at(2).unwrap(), b"c");
}

#[test]
fn unicode_column_counts_utf8_bytes() {
    let bytes = "αβ\n".as_bytes();
    let snapshot = MemorySnapshot::from_bytes(
        SnapshotRef::new(SourceRef::new("oak", "unicode.nd").unwrap()),
        bytes.to_vec(),
    );
    let coords = snapshot.text_view().coords_at_offset(3).unwrap();
    assert_eq!(coords.line, 0);
    assert_eq!(coords.column, 3);
}

#[test]
fn partial_access_reports_need_range() {
    let bytes = b"abcdef";
    let partial = PartialByteAccess::new(bytes, 4).unwrap();
    let err = partial.read(ByteRange::new(0, 6).unwrap()).unwrap_err();
    assert!(matches!(err, AccessError::NeedRange { .. }));
}

#[test]
fn invalid_snapshot_handle_does_not_resolve() {
    let store = MemoryStore::new();
    assert!(matches!(store.get(SnapshotHandle::INVALID), Err(AccessError::SnapshotReleased)));
}

#[test]
fn released_snapshot_handle_fails() {
    let mut store = MemoryStore::new();
    let snapshot = MemorySnapshot::from_bytes(
        SnapshotRef::with_revision("oak", "sample.nd", "1").unwrap(),
        Vec::new(),
    );
    let handle = store.insert(snapshot);
    store.release(handle);
    assert!(matches!(store.get(handle), Err(AccessError::SnapshotReleased)));
}

#[test]
fn different_revisions_produce_different_snapshot_refs() {
    let left = SnapshotRef::with_revision("oak", "sample.nd", "1").unwrap();
    let right = SnapshotRef::with_revision("oak", "sample.nd", "2").unwrap();
    assert_ne!(left, right);
}

#[test]
fn wire_round_trip_preserves_special_characters() {
    let source = SourceRef::new("oak", "zip|member").unwrap().with_revision("rev\\1").unwrap();
    let wire = encode_source_ref(&source);
    let decoded = decode_source_ref(&wire).unwrap();
    assert_eq!(decoded, source);
}

#[test]
fn serde_round_trip_validates_byte_range() {
    let range = ByteRange::new(1, 4).unwrap();
    let json = serde_json::to_string(&range).unwrap();
    let decoded: ByteRange = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, range);

    let invalid = r#"{"start":9,"end":4}"#;
    assert!(serde_json::from_str::<ByteRange>(invalid).is_err());
}

#[test]
fn text_view_reads_utf8_range() {
    let snapshot = MemorySnapshot::from_bytes(
        SnapshotRef::new(SourceRef::new("oak", "sample.nd").unwrap()),
        "αβ".as_bytes().to_vec(),
    );
    let view = snapshot.text_view();
    assert_eq!(view.utf8_range(ByteRange::new(0, 4).unwrap()).unwrap(), "αβ");
}
