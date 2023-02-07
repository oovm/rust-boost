//! Legacy terminal rendering for text diagnostics.

mod characters;
mod display;
mod draw;
mod style;
mod windows;
mod write;

use characters::Draw;
use display::*;
pub use characters::{BuiltinDrawer, DrawElements};
pub use draw::{Console, Palette};
pub use style::{color::Color, paint::Paint, style::Style};
pub use source_cache::{SourceCache, SourceID, SourceSpan};
pub use windows::enable_ansi_color;

use core::cmp::{Eq, PartialEq};
use core::fmt::{Debug, Display, Formatter};
use core::hash::Hash;
use std::io::Write;
use unicode_width::UnicodeWidthChar;

/// A type that represents a labelled section of identifier code.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Label {
    span: SourceSpan,
    msg: Option<String>,
    color: Option<Color>,
    order: i32,
    priority: i32,
}

impl Label {
    /// Create a new [`Label`].
    pub fn new(span: SourceSpan) -> Self {
        Self { span, msg: None, color: None, order: 0, priority: 0 }
    }

    /// Give this label a message.
    pub fn with_message<M: ToString>(mut self, msg: M) -> Self {
        self.msg = Some(msg.to_string());
        self
    }

    /// Give this label a highlight colour.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Specify the order of this label relative to other labels.
    ///
    /// Lower values correspond to this label having an earlier order.
    ///
    /// If unspecified, labels default to an order of `0`.
    ///
    /// When labels are displayed after a line the crate needs to decide which labels should be displayed first. By
    /// Default, the orders labels based on where their associated line meets the text (see [`LabelAttach`]).
    /// Additionally, multi-line labels are ordered before inline labels. You can use this function to override this
    /// behaviour.
    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    /// Specify the priority of this label relative to other labels.
    ///
    /// Higher values correspond to this label having a higher priority.
    ///
    /// If unspecified, labels default to a priority of `0`.
    ///
    /// Label spans can overlap. When this happens, the crate needs to decide which labels to prioritise for various
    /// purposes such as highlighting. By default, spans with a smaller length get a higher priority. You can use this
    /// function to override this behaviour.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// A type representing a diagnostic that is ready to be written to output.
pub struct Diagnostic {
    kind: Box<dyn ReportLevel>,
    code: Option<usize>,
    message: String,
    note: Option<String>,
    help: Option<String>,
    file: SourceID,
    location: Option<u32>,
    labels: Vec<Label>,
    config: Config,
}

/// A type used to build a [`Diagnostic`].
pub struct DiagnosticBuilder {
    inner: Diagnostic,
}

impl Diagnostic {
    /// Begin building a new [`Diagnostic`].
    pub fn new<R>(kind: R) -> DiagnosticBuilder
    where
        R: ReportLevel + 'static,
    {
        DiagnosticBuilder {
            inner: Diagnostic {
                kind: Box::new(kind),
                code: None,
                message: "".to_string(),
                note: None,
                help: None,
                file: Default::default(),
                location: None,
                labels: vec![],
                config: Default::default(),
            },
        }
    }
}
impl Diagnostic {
    /// Write this diagnostic out to `stderr`.
    pub fn eprint(&self, cache: &SourceCache) -> std::io::Result<()> {
        self.write(cache, std::io::stderr().lock())
    }

    /// Write this diagnostic out to `stdout`.
    ///
    /// In most cases, [`Diagnostic::eprint`] is the
    /// ['more correct'](https://en.wikipedia.org/wiki/Standard_streams#Standard_error_(stderr)) function to use.
    pub fn print(&self, cache: &SourceCache) -> std::io::Result<()> {
        self.write_for_stdout(cache, std::io::stdout().lock())
    }
}

impl Debug for Diagnostic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Report")
            .field("kind", &self.kind)
            .field("code", &self.code)
            .field("msg", &self.message)
            .field("note", &self.note)
            .field("help", &self.help)
            .field("config", &self.config)
            .finish()
    }
}

/// A builder for [`Diagnostic`].
pub trait ReportLevel: Debug {
    /// The level of this report.
    fn level(&self) -> u8;
    /// The color of this report.
    fn get_color(&self) -> Color;
}

impl Debug for ReportKind {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ReportKind::Error => f.write_str("ERROR"),
            ReportKind::Alert => f.write_str("ALERT"),
            ReportKind::Trace => f.write_str("TRACE"),
            ReportKind::Blame => f.write_str("BLAME"),
            ReportKind::Fatal => f.write_str("FATAL"),
        }
    }
}

impl ReportLevel for ReportKind {
    fn level(&self) -> u8 {
        match self {
            ReportKind::Trace => 0,
            ReportKind::Blame => 150,
            ReportKind::Alert => 200,
            ReportKind::Error => 250,
            ReportKind::Fatal => 255,
        }
    }

    fn get_color(&self) -> Color {
        match self {
            ReportKind::Trace => Color::Cyan,
            ReportKind::Blame => Color::Green,
            ReportKind::Alert => Color::Yellow,
            ReportKind::Error => Color::Red,
            ReportKind::Fatal => Color::Magenta,
        }
    }
}

/// @trace 0
/// @print 100
/// @blame 150
/// @risky 175
/// @alert 200
/// @error 250
/// @fatal 255
/// A type that defines the kind of report being produced.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ReportKind {
    /// The report is advice to the user about a potential anti-pattern of other benign issues.
    Trace,
    /// The report is advice to the user about a potential anti-pattern of other benign issues.
    Blame,
    /// The report is a warning and indicates a likely problem, but not to the extent that the requested action cannot
    /// be performed.
    Alert,
    /// The report is an error and indicates a critical problem that prevents the program performing the requested
    /// action.
    Error,
    /// Fatal error that caused this program to terminate
    Fatal,
}

impl DiagnosticBuilder {
    /// Set the kind of this report.
    pub fn set_code(&mut self, code: Option<usize>) {
        self.inner.code = code;
    }

    /// Give this report a numerical code that may be used to more precisely look up the error in documentation.
    pub fn with_code(mut self, code: usize) -> Self {
        self.set_code(Some(code));
        self
    }

    /// Set the message of this report.
    pub fn set_message<M: ToString>(&mut self, message: M) {
        self.inner.message = message.to_string();
    }

    /// Add a message to this report.
    pub fn with_message<M: ToString>(mut self, message: M) -> Self {
        self.inner.message = message.to_string();
        self
    }
    /// Set the source_text of this report.
    pub fn set_location(&mut self, file: SourceID, start: Option<u32>) {
        self.inner.file = file;
        self.inner.location = start;
    }
    /// Set the source_text of this report.
    pub fn with_location(mut self, file: SourceID, start: Option<u32>) -> Self {
        self.set_location(file, start);
        self
    }

    /// Set the note of this report.
    pub fn set_note<N: ToString>(&mut self, note: N) {
        self.inner.note = Some(note.to_string());
    }

    /// Set the note of this report.
    pub fn with_note<N: ToString>(mut self, note: N) -> Self {
        self.set_note(note);
        self
    }

    /// Set the help message of this report.
    pub fn set_help<N: ToString>(&mut self, note: N) {
        self.inner.help = Some(note.to_string());
    }

    /// Set the help message of this report.
    pub fn with_help<N: ToString>(mut self, note: N) -> Self {
        self.set_help(note);
        self
    }

    /// Add a label to the report.
    pub fn add_label(&mut self, label: Label) {
        self.add_labels(std::iter::once(label));
    }

    /// Add multiple labels to the report.
    pub fn add_labels<L: IntoIterator<Item = Label>>(&mut self, labels: L) {
        let config = &self.inner.config; // This would not be necessary in Rust 2021 edition
        self.inner.labels.extend(labels.into_iter().map(|mut label| {
            label.color = config.filter_color(label.color);
            label
        }));
    }

    /// Add a label to the report.
    pub fn with_label(mut self, label: Label) -> Self {
        self.add_label(label);
        self
    }

    /// Add multiple labels to the report.
    pub fn with_labels<L: IntoIterator<Item = Label>>(mut self, labels: L) -> Self {
        self.add_labels(labels);
        self
    }

    /// Use the given [`Config`] to determine diagnostic attributes.
    pub fn with_config(mut self, config: Config) -> Self {
        self.inner.config = config;
        self
    }

    /// Finish building the [`Diagnostic`].
    pub fn finish(self) -> Diagnostic {
        self.inner
    }
}

impl Debug for DiagnosticBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

/// The attachment point of inline label arrows
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LabelAttach {
    /// Arrows should attach to the start of the label source_text.
    Start,
    /// Arrows should attach to the middle of the label source_text (or as close to the middle as we can get).
    Middle,
    /// Arrows should attach to the end of the label source_text.
    End,
}

/// A type used to configure a report
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Config {
    cross_gap: bool,
    label_attach: LabelAttach,
    compact: bool,
    underlines: bool,
    multiline_arrows: bool,
    /// is enable
    pub color_enable: bool,
    /// custom margin color
    pub margin_color: Option<Color>,
    /// custom margin skip color
    pub margin_skip_color: Option<Color>,
    /// custom important
    pub unimportant_color: Option<Color>,
    tab_width: usize,
    /// Custom character sets
    pub characters: DrawElements,
}

impl Config {
    /// When label lines cross one-another, should there be a gap?
    ///
    /// The alternative to this is to insert crossing characters. However, these interact poorly with label colours.
    ///
    /// If unspecified, this defaults to [`false`].
    pub fn with_cross_gap(mut self, cross_gap: bool) -> Self {
        self.cross_gap = cross_gap;
        self
    }
    /// Where should inline labels attach to their spans?
    ///
    /// If unspecified, this defaults to [`LabelAttach::Middle`].
    pub fn with_label_attach(mut self, label_attach: LabelAttach) -> Self {
        self.label_attach = label_attach;
        self
    }
    /// Should the report remove gaps to minimise used space?
    ///
    /// If unspecified, this defaults to [`false`].
    pub fn with_compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
    /// Should underlines be used for label source_text where possible?
    ///
    /// If unspecified, this defaults to [`true`].
    pub fn with_underlines(mut self, underlines: bool) -> Self {
        self.underlines = underlines;
        self
    }
    /// Should arrows be used to point to the bounds of multi-line spans?
    ///
    /// If unspecified, this defaults to [`true`].
    pub fn with_multiline_arrows(mut self, multiline_arrows: bool) -> Self {
        self.multiline_arrows = multiline_arrows;
        self
    }
    /// Should colored output should be enabled?
    ///
    /// If unspecified, this defaults to [`true`].
    pub fn with_color(mut self, color: bool) -> Self {
        self.color_enable = color;
        self
    }
    /// How many characters width should tab characters be?
    ///
    /// If unspecified, this defaults to `4`.
    pub fn with_tab_width(mut self, tab_width: usize) -> Self {
        self.tab_width = tab_width;
        self
    }
    /// What character set should be used to display dynamic elements such as boxes and arrows?
    ///
    /// If unspecified, this defaults to [`BuiltinDrawer::Unicode`].
    pub fn with_characters(mut self, set: impl Draw) -> Self {
        self.characters = set.get_elements();
        self
    }

    fn margin_color(&self) -> Option<Color> {
        Some(match self.margin_color {
            None => Color::Fixed(27),
            Some(s) => s,
        })
        .filter(|_| self.color_enable)
    }
    fn skipped_margin_color(&self) -> Option<Color> {
        Some(match self.margin_skip_color {
            None => Color::Fixed(27),
            Some(s) => s,
        })
        .filter(|_| self.color_enable)
    }
    fn unimportant_color(&self) -> Option<Color> {
        Some(match self.unimportant_color {
            None => Color::Fixed(249),
            Some(s) => s,
        })
        .filter(|_| self.color_enable)
    }
    fn note_color(&self) -> Option<Color> {
        Some(Color::Fixed(115)).filter(|_| self.color_enable)
    }
    fn filter_color(&self, color: Option<Color>) -> Option<Color> {
        color.filter(|_| self.color_enable)
    }

    // Find the character that should be drawn and the number of times it should be drawn for each char
    fn char_width(&self, c: char, col: usize) -> (char, usize) {
        match c {
            '\t' => {
                // Find the column that the tab should end at
                let tab_end = (col / self.tab_width + 1) * self.tab_width;
                (' ', tab_end - col)
            }
            c if c.is_whitespace() => (' ', 1),
            _ => (c, c.width().unwrap_or(1)),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cross_gap: true,
            label_attach: LabelAttach::Middle,
            compact: false,
            underlines: true,
            multiline_arrows: true,
            color_enable: true,
            margin_color: None,
            margin_skip_color: None,
            unimportant_color: None,
            tab_width: 4,
            characters: BuiltinDrawer::Unicode.get_elements(),
        }
    }
}
