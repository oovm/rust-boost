use diagnostic::{
    AddressSpaceRef, ByteRange, Diagnostic, DiagnosticCode, DiagnosticLabel, DiagnosticLocation, DiagnosticOrigin,
    DocumentRef, LabelRole, MappingPrecision, MemberPath, MemberSegment, Message, ObjectRef, SemanticPath, SourceRef,
};
use diagnostic::{DiagnosticSeverity, DiagnosticSet};

pub fn text_diagnostic() -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::new("oak.syntax.unexpected-token"),
        DiagnosticSeverity::Error,
        DiagnosticOrigin::new("oak", "xml").with_stage("parse"),
        Message::new("oak.syntax.unexpected-token").with_fallback("unexpected token"),
    )
    .with_primary(DiagnosticLabel::new(
        DiagnosticLocation::Text {
            source: SourceRef::new("oak", "sample.xml").with_revision("1"),
            range: ByteRange::new(12, 13).unwrap(),
        },
        Message::new("label.token").with_fallback("token"),
        LabelRole::Primary,
    ))
}

pub fn binary_diagnostic() -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::new("acorn.layout.offset-out-of-range"),
        DiagnosticSeverity::Error,
        DiagnosticOrigin::new("acorn", "bin"),
        Message::new("acorn.layout.offset-out-of-range").with_fallback("offset out of range"),
    )
    .with_primary(DiagnosticLabel::new(
        DiagnosticLocation::Binary {
            source: SourceRef::new("acorn", "firmware.bin"),
            address_space: AddressSpaceRef::new("acorn", "file"),
            range: ByteRange::new(0x1000, 0x1004).unwrap(),
        },
        Message::new("label.binary").with_fallback("binary"),
        LabelRole::Primary,
    ))
}

pub fn member_diagnostic() -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::new("acorn.container.need-range"),
        DiagnosticSeverity::Warning,
        DiagnosticOrigin::new("acorn", "zip"),
        Message::new("acorn.container.need-range").with_fallback("need range"),
    )
    .with_primary(DiagnosticLabel::new(
        DiagnosticLocation::Member {
            container: SourceRef::new("acorn", "docx.zip"),
            member: MemberPath::new(vec![
                MemberSegment::new("zip", "word/document.xml"),
                MemberSegment::new("view", "utf8"),
            ]),
            range: Some(ByteRange::new(128, 140).unwrap()),
            precision: MappingPrecision::Container,
        },
        Message::new("label.member").with_fallback("member"),
        LabelRole::Primary,
    ))
}

pub fn object_diagnostic() -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::new("acorn.layout.offset-out-of-range"),
        DiagnosticSeverity::Error,
        DiagnosticOrigin::new("acorn", "pdf"),
        Message::new("acorn.layout.offset-out-of-range").with_fallback("offset out of range"),
    )
    .with_primary(DiagnosticLabel::new(
        DiagnosticLocation::Object {
            source: SourceRef::new("acorn", "sample.pdf"),
            object: ObjectRef::new("xref", "42"),
        },
        Message::new("label.object").with_fallback("object"),
        LabelRole::Primary,
    ))
}

pub fn semantic_diagnostic() -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::new("notedown.semantic.unresolved-target"),
        DiagnosticSeverity::Error,
        DiagnosticOrigin::new("notedown", "ir"),
        Message::new("notedown.semantic.unresolved-target").with_fallback("unresolved target"),
    )
    .with_primary(DiagnosticLabel::new(
        DiagnosticLocation::Semantic {
            document: DocumentRef::new("notedown", "chapter-1"),
            path: SemanticPath::new("/blocks/17"),
        },
        Message::new("label.semantic").with_fallback("semantic"),
        LabelRole::Primary,
    ))
}

pub fn sample_set() -> DiagnosticSet {
    let mut set = DiagnosticSet::new();
    set.push(text_diagnostic());
    set.push(binary_diagnostic());
    set.push(member_diagnostic());
    set.push(object_diagnostic());
    set.push(semantic_diagnostic());
    set
}
