use diagnostic::{Color, Console, Palette};
use diagnostic::terminal::{enable_ansi_color, Config, Diagnostic, Label, ReportKind, SourceID};
use source_cache::{SourceCache, SourceText};
use std::{iter::zip, ops::Range};

mod multi_file;
mod multi_line;
mod stress_test;

fn debug_lines(lines: Vec<&str>) {
    let source: String = lines.iter().map(|s| *s).collect();
    let source = SourceText::from(source);

    assert_eq!(source.lines().len(), lines.len());

    let mut offset = 0;
    for (source_line, raw_line) in zip(source.lines().into_iter(), lines.into_iter()) {
        assert_eq!(source_line.offset as usize, offset);
        assert_eq!(source_line.length as usize, raw_line.len());
        assert_eq!(source_line.text, raw_line.trim_end());
        offset += source_line.length as usize;
    }

    assert_eq!(source.get_length(), offset);
}

#[test]
fn simple() {
    let mut files = SourceCache::default();
    let sample = files.load_text(include_str!("sample.tao"), "sample.tao");

    Diagnostic::new(ReportKind::Blame)
        .with_location(sample, Some(12))
        .with_message("Incompatible types")
        .with_code(12)
        .with_label(Label::new(sample.with_range(32..33)).with_message("This is of type Nat"))
        .with_label(Label::new(sample.with_range(42..45)).with_message("This is of type Str"))
        .finish()
        .print(&files)
        .unwrap();
}

#[test]
fn source_from() {
    debug_lines(vec![]); // Empty string

    debug_lines(vec!["Single line"]);
    debug_lines(vec!["Single line with LF\n"]);
    debug_lines(vec!["Single line with CRLF\r\n"]);

    debug_lines(vec!["Two\r\n", "lines\n"]);
    debug_lines(vec!["Some\n", "more\r\n", "lines"]);
    debug_lines(vec!["\n", "\r\n", "\n", "Empty Lines"]);

    debug_lines(vec!["Trailing spaces  \n", "are trimmed\t"]);

    // Line endings other than LF or CRLF
    debug_lines(vec!["CR\r", "VT\x0B", "FF\x0C", "NEL\u{0085}", "LS\u{2028}", "PS\u{2029}"]);
}
