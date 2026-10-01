#![deny(missing_docs)]
//! Renders the preview SVG for the README.
//!
//! To update the preview, execute the following command from the top level of
//! the repository:
//!
//! ```sh
//! cargo run --example readme_preview svg > labels/assets/readme_preview.svg
//! ```

use std::io::{self, Write};

use structopt::StructOpt;

use diagnostic::{
    diagnostic::{Diagnostic, Label},
    term::{
        self,
        termcolor::{Color, ColorSpec, StandardStream, WriteColor},
        ColorArg, Styles, TerminalConfig,
    },
    Diagnostic, DiagnosticResult, Label, TextStorage,
};

#[derive(Debug, StructOpt)]
#[structopt(name = "emit")]
pub enum Opts {
    /// Render SVG output
    Svg,
    /// Render Stderr output
    Stderr {
        /// Configure coloring of output
        #[structopt(
        long = "color",
        parse(try_from_str),
        default_value = "auto",
        possible_values = ColorArg::VARIANTS,
        case_insensitive = true
        )]
        color: ColorArg,
    },
}

fn main() -> DiagnosticResult {
    let mut store = TextStorage::default();

    store.anonymous(
        "FizzBuzz.fun",
        unindent::unindent(
            r#"
                module FizzBuzz where

                fizz₁ : Nat → String
                fizz₁ num = case (mod num 5) (mod num 3) of
                    0 0 => "FizzBuzz"
                    0 _ => "Fizz"
                    _ 0 => "Buzz"
                    _ _ => num

                fizz₂ : Nat → String
                fizz₂ num =
                    case (mod num 5) (mod num 3) of
                        0 0 => "FizzBuzz"
                        0 _ => "Fizz"
                        _ 0 => "Buzz"
                        _ _ => num
            "#,
        ),
    );

    let diagnostics = [Diagnostic::error()
        .with_message("`case` clauses have incompatible types")
        .with_code("E0308")
        .with_labels(vec![
            Label::primary("GG".to_string(), 328..331, "expected `String`, found `Nat`"),
            Label::secondary("GG".to_string(), 211..331, "`case` clauses have incompatible types"),
            Label::secondary("GG".to_string(), 258..268, "this is found to be of type `String`"),
            Label::secondary("GG".to_string(), 284..290, "this is found to be of type `String`"),
            Label::secondary("GG".to_string(), 306..312, "this is found to be of type `String`"),
            Label::secondary("GG".to_string(), 186..192, "expected type `String` found here"),
        ])
        .with_notes(vec![unindent::unindent(
            "
                expected type `String`
                   found type `Nat`
            ",
        )])];

    // let mut errors = SimpleFiles::new();
    match Opts::from_args() {
        Opts::Svg => {
            let mut buffer = Vec::new();
            let mut writer = HtmlEscapeWriter::new(SvgWriter::new(&mut buffer));
            let config = TerminalConfig { styles: Styles::with_color(Color::Blue), ..TerminalConfig::default() };

            for diagnostic in &diagnostics {
                term::emit(&mut writer, &config, &store, diagnostic)?;
            }

            let num_lines = buffer.iter().filter(|byte| **byte == b'\n').count() + 1;

            let padding = 10;
            let font_size = 12;
            let line_spacing = 3;
            let width = 882;
            let height = padding + num_lines * (font_size + line_spacing) + padding;

            let stdout = std::io::stdout();
            let writer = &mut stdout.lock();

            write!(
                writer,
                r#"<svg viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg">
  <style>
    /* https://github.com/aaron-williamson/base16-alacritty/blob/master/colors/base16-tomorrow-night-256.yml */
    pre {{
      background: #1d1f21;
      margin: 0;
      padding: {padding}px;
      border-radius: 6px;
      color: #ffffff;
      font: {font_size}px SFMono-Regular, Consolas, Liberation Mono, Menlo, monospace;
    }}

    pre .bold {{ font-weight: bold; }}

    pre .fg.black   {{ color: #1d1f21; }}
    pre .fg.red     {{ color: #cc6666; }}
    pre .fg.green   {{ color: #b5bd68; }}
    pre .fg.yellow  {{ color: #f0c674; }}
    pre .fg.blue    {{ color: #81a2be; }}
    pre .fg.magenta {{ color: #b294bb; }}
    pre .fg.cyan    {{ color: #8abeb7; }}
    pre .fg.white   {{ color: #c5c8c6; }}

    pre .fg.black.bright    {{ color: #969896; }}
    pre .fg.red.bright      {{ color: #cc6666; }}
    pre .fg.green.bright    {{ color: #b5bd68; }}
    pre .fg.yellow.bright   {{ color: #f0c674; }}
    pre .fg.blue.bright     {{ color: #81a2be; }}
    pre .fg.magenta.bright  {{ color: #b294bb; }}
    pre .fg.cyan.bright     {{ color: #8abeb7; }}
    pre .fg.white.bright    {{ color: #ffffff; }}

    pre .bg.black   {{ background-color: #1d1f21; }}
    pre .bg.red     {{ background-color: #cc6666; }}
    pre .bg.green   {{ background-color: #b5bd68; }}
    pre .bg.yellow  {{ background-color: #f0c674; }}
    pre .bg.blue    {{ background-color: #81a2be; }}
    pre .bg.magenta {{ background-color: #b294bb; }}
    pre .bg.cyan    {{ background-color: #8abeb7; }}
    pre .bg.white   {{ background-color: #c5c8c6; }}

    pre .bg.black.bright    {{ background-color: #969896; }}
    pre .bg.red.bright      {{ background-color: #cc6666; }}
    pre .bg.green.bright    {{ background-color: #b5bd68; }}
    pre .bg.yellow.bright   {{ background-color: #f0c674; }}
    pre .bg.blue.bright     {{ background-color: #81a2be; }}
    pre .bg.magenta.bright  {{ background-color: #b294bb; }}
    pre .bg.cyan.bright     {{ background-color: #8abeb7; }}
    pre .bg.white.bright    {{ background-color: #ffffff; }}
  </style>

  <foreignObject x="0" y="0" width="{width}" height="{height}">
    <div xmlns="http://www.w3.org/1999/xhtml">
      <pre>"#,
                padding = padding,
                font_size = font_size,
                width = width,
                height = height,
            )?;

            writer.write_all(&buffer)?;

            write!(
                writer,
                "</pre>
    </div>
  </foreignObject>
</svg>
"
            )?;
        }
        Opts::Stderr { color } => {
            let writer = StandardStream::stderr(color.into());
            let config = TerminalConfig::default();
            for diagnostic in &diagnostics {
                term::emit(&mut writer.lock(), &config, &store, diagnostic)?;
            }
        }
    }

    Ok(())
}

/// Rudimentary HTML escaper which performs the following conversions:
///
/// - `<` ⇒ `&lt;`
/// - `>` ⇒ `&gt;`
/// - `&` ⇒ `&amp;`
pub struct HtmlEscapeWriter<W> {
    upstream: W,
}

impl<W> HtmlEscapeWriter<W> {
    pub fn new(upstream: W) -> HtmlEscapeWriter<W> {
        HtmlEscapeWriter { upstream }
    }
}

impl<W: Write> Write for HtmlEscapeWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut last_term = 0usize;
        for (i, byte) in buf.iter().enumerate() {
            let escape = match byte {
                b'<' => &b"&lt;"[..],
                b'>' => &b"&gt;"[..],
                b'&' => &b"&amp;"[..],
                _ => continue,
            };
            self.upstream.write_all(&buf[last_term..i])?;
            last_term = i + 1;
            self.upstream.write_all(escape)?;
        }
        self.upstream.write_all(&buf[last_term..])?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.upstream.flush()
    }
}

impl<W: WriteColor> WriteColor for HtmlEscapeWriter<W> {
    fn supports_color(&self) -> bool {
        self.upstream.supports_color()
    }

    fn set_color(&mut self, spec: &ColorSpec) -> io::Result<()> {
        self.upstream.set_color(spec)
    }

    fn reset(&mut self) -> io::Result<()> {
        self.upstream.reset()
    }
}

pub struct SvgWriter<W> {
    upstream: W,
    color: ColorSpec,
}

impl<W> SvgWriter<W> {
    pub fn new(upstream: W) -> SvgWriter<W> {
        SvgWriter { upstream, color: ColorSpec::new() }
    }
}

impl<W: Write> Write for SvgWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.upstream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.upstream.flush()
    }
}

impl<W: Write> WriteColor for SvgWriter<W> {
    fn supports_color(&self) -> bool {
        true
    }

    fn set_color(&mut self, spec: &ColorSpec) -> io::Result<()> {
        #![allow(unused_assignments)]

        if self.color == *spec {
            return Ok(());
        }
        else {
            if !self.color.is_none() {
                write!(self, "</source_text>")?;
            }
            self.color = spec.clone();
        }

        if spec.is_none() {
            write!(self, "</source_text>")?;
            return Ok(());
        }
        else {
            write!(self, "<source_text class=\"")?;
        }

        let mut first = true;

        fn write_first<W: Write>(first: bool, writer: &mut SvgWriter<W>) -> io::Result<bool> {
            if !first {
                write!(writer, " ")?;
            }

            Ok(false)
        }

        fn write_color<W: Write>(color: &Color, writer: &mut SvgWriter<W>) -> io::Result<()> {
            match color {
                Color::Black => write!(writer, "black"),
                Color::Blue => write!(writer, "blue"),
                Color::Green => write!(writer, "green"),
                Color::Red => write!(writer, "red"),
                Color::Cyan => write!(writer, "cyan"),
                Color::Magenta => write!(writer, "magenta"),
                Color::Yellow => write!(writer, "yellow"),
                Color::White => write!(writer, "white"),
                // TODO: other colors
                _ => Ok(()),
            }
        }

        if let Some(fg) = spec.fg() {
            first = write_first(first, self)?;
            write!(self, "fg ")?;
            write_color(fg, self)?;
        }

        if let Some(bg) = spec.bg() {
            first = write_first(first, self)?;
            write!(self, "bg ")?;
            write_color(bg, self)?;
        }

        if spec.bold() {
            first = write_first(first, self)?;
            write!(self, "bold")?;
        }

        if spec.underline() {
            first = write_first(first, self)?;
            write!(self, "underline")?;
        }

        if spec.intense() {
            first = write_first(first, self)?;
            write!(self, "bright")?;
        }

        write!(self, "\">")?;

        Ok(())
    }

    fn reset(&mut self) -> io::Result<()> {
        let color = self.color.clone();

        if color != ColorSpec::new() {
            write!(self, "</source_text>")?;
            self.color = ColorSpec::new();
        }

        Ok(())
    }
}
