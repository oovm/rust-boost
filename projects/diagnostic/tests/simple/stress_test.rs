use super::*;

#[test]
fn main() {
    let mut files = SourceCache::default();
    let stress = files.load_text(include_str!("stresstest.tao"), "stresstest.tao");

    let mut colors = Palette::new();

    Diagnostic::new(ReportKind::Error)
        .with_location(stress, Some(13))
        .with_code(3)
        .with_message(format!("Incompatible types"))
        .with_label(Label::new(stress.with_range(0..1)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(1..2)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(2..3)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(3..4)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(4..5)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(5..6)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(6..7)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(7..8)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(8..9)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(9..10)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(10..11)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(11..12)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(12..13)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(13..14)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(14..15)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(15..16)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(16..17)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(17..18)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(18..19)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(19..20)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(20..21)).with_message("Color").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(18..19)).with_message("This is of type Nat").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(13..16)).with_message("This is of type Str").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(40..41)).with_message("This is of type Nat").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(43..47)).with_message("This is of type Bool").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(49..51)).with_message("This is of type ()").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(53..55)).with_message("This is of type [_]").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(25..78)).with_message("This is of type Str").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(81..124)).with_message("This is of type Nat").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(100..126)).with_message("This is an inner multi-line").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(106..120)).with_message("This is another inner multi-line").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(108..122)).with_message("This is *really* nested multi-line").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(110..111)).with_message("This is an inline within the nesting!").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(111..112)).with_message("And another!").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(103..123)).with_message("This is *really* nested multi-line").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(105..125)).with_message("This is *really* nested multi-line").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(112..116)).with_message("This is *really* nested multi-line").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(26..100)).with_message("Hahaha!").with_color(Color::Fixed(75)))
        .with_label(Label::new(stress.with_range(85..110)).with_message("Oh god, no more 1").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(84..114)).with_message("Oh god, no more 2").with_color(colors.random()))
        .with_label(Label::new(stress.with_range(89..113)).with_message("Oh god, no more 3").with_color(colors.random()))
        .with_config(Config::default().with_cross_gap(false).with_compact(true).with_underlines(true).with_tab_width(4))
        .finish()
        .print(&files)
        .unwrap();
}
