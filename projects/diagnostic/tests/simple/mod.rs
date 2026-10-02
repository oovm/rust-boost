use diagnostic::{Color, Console, Palette};
use diagnostic::terminal::{enable_ansi_color, Config, Diagnostic, Label, ReportKind, SourceCache};

mod multi_file;
mod multi_line;
mod stress_test;

fn debug_lines(lines: Vec<&str>) {
    let mut cache = SourceCache::default();
    let source: String = lines.iter().map(|s| *s).collect();
    let id = cache.load_text(source, "snippet");
    let view = cache.fetch(&id).expect("snippet must resolve");

    let expected_lines = if lines.is_empty() { 1 } else { lines.len() };
    assert_eq!(view.line_count(), expected_lines);

    for (index, raw_line) in lines.iter().enumerate() {
        let source_line = view.get_line(index).expect("line must exist");
        let expected = raw_line.trim_end_matches(|ch| matches!(ch, '\n' | '\r'));
        assert_eq!(source_line.text, expected);
    }
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
    debug_lines(vec![]);

    debug_lines(vec!["Single line"]);
    debug_lines(vec!["Single line with LF\n"]);
    debug_lines(vec!["Single line with CRLF\r\n"]);

    debug_lines(vec!["Two\r\n", "lines\n"]);
    debug_lines(vec!["Some\n", "more\r\n", "lines"]);
    debug_lines(vec!["\n", "\r\n", "\n", "Empty Lines"]);

    debug_lines(vec!["Trailing spaces  \n", "are trimmed\t"]);
}
