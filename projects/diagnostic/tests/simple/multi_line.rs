use super::*;

#[test]
fn main() {
    let mut colors = Palette::new();

    // Generate & choose some colours for each of our elements
    let a = colors.random();
    let b = colors.random();
    let out = Color::Fixed(81);
    let out2 = colors.random();

    let mut files = SourceCache::default();
    let sample = files.load_text(include_str!("sample.tao"), "sample.tao");

    Diagnostic::new(ReportKind::Error)
        .with_location(sample, Some(12))
        .with_code(3)
        .with_message(format!("Incompatible types"))
        .with_label(Label::new(sample.with_range(32..33)).with_message(format!("This is of type {}", "Nat".fg(a))).with_color(a))
        .with_label(Label::new(sample.with_range(42..45)).with_message(format!("This is of type {}", "Str".fg(b))).with_color(b))
        .with_label(
            Label::new(sample.with_range(11..48))
                .with_message(format!("The values are outputs of this {} expression", "match".fg(out),))
                .with_color(out),
        )
        .with_label(Label::new(sample.with_range(0..48)).with_message(format!("The {} has a problem", "definition".fg(out2),)).with_color(out2))
        .with_label(Label::new(sample.with_range(50..76)).with_message(format!("Usage of {} here", "definition".fg(out2),)).with_color(out2))
        .with_note(format!("Outputs of {} expressions must coerce to the same type", "match".fg(out)))
        .finish()
        .print(&files)
        .unwrap();
}
