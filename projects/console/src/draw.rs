use std::fmt::{Display, Formatter};

use crate::{Color, Paint};

/// Output stream to check for whether color is enabled.
#[derive(Clone, Copy, Debug)]
pub enum StreamType {
    /// Standard Output
    Stdout,
    /// Standard Error
    Stderr,
}

/// A trait used to add formatting attributes to displayable items intended to be written to a
/// particular stream (`stdout` or `stderr`).
///
/// Attributes specified through this trait are not composable (i.e: the behaviour of two nested attributes each with a
/// conflicting attribute is left unspecified).
pub trait StreamAwareFmt: Sized {
    #[doc(hidden)]
    fn color_enabled_for(_: StreamType) -> bool {
        true
    }

    /// Give this value the specified foreground colour, when color is enabled for the specified stream.
    fn fg<C: Into<Option<Color>>>(self, color: C, stream: StreamType) -> Foreground<Self> {
        if Self::color_enabled_for(stream) { Foreground(self, color.into()) } else { Foreground(self, None) }
    }

    /// Give this value the specified background colour, when color is enabled for the specified stream.
    fn bg<C: Into<Option<Color>>>(self, color: C, stream: StreamType) -> Background<Self> {
        if Self::color_enabled_for(stream) { Background(self, color.into()) } else { Background(self, None) }
    }
}

impl<T: Display> StreamAwareFmt for T {}

/// A trait used to add formatting attributes to displayable items.
///
/// If using the `concolor` feature, this trait assumes that the items are going to be printed to
/// `stderr`. If you are printing to `stdout`, `use` the [`StdoutFmt`] trait instead.
///
/// Attributes specified through this trait are not composable (i.e: the behaviour of two nested attributes each with a
/// conflicting attribute is left unspecified).
pub trait Console: Sized {
    /// Give this value the specified foreground colour.
    fn fg<C: Into<Option<Color>>>(self, color: C) -> Foreground<Self>
    where
        Self: Display,
    {
        if cfg!(feature = "concolor") {
            StreamAwareFmt::fg(self, color, StreamType::Stderr)
        }
        else {
            Foreground(self, color.into())
        }
    }

    /// Give this value the specified background colour.
    fn bg<C: Into<Option<Color>>>(self, color: C) -> Background<Self>
    where
        Self: Display,
    {
        if cfg!(feature = "concolor") {
            StreamAwareFmt::bg(self, color, StreamType::Stdout)
        }
        else {
            Background(self, color.into())
        }
    }
}

impl<T: Display> Console for T {}

/// Foreground color wrapper for stream-aware formatting.
#[derive(Copy, Clone, Debug)]
pub struct Foreground<T>(T, Option<Color>);
impl<T: Display> Display for Foreground<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if let Some(col) = self.1 { write!(f, "{}", Paint::new(&self.0).fg(col)) } else { write!(f, "{}", self.0) }
    }
}

/// Background color wrapper for stream-aware formatting.
#[derive(Copy, Clone, Debug)]
pub struct Background<T>(T, Option<Color>);
impl<T: Display> Display for Background<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if let Some(col) = self.1 { write!(f, "{}", Paint::new(&self.0).bg(col)) } else { write!(f, "{}", self.0) }
    }
}

/// A type that can generate distinct 8-bit colors.
pub struct Palette {
    state: [u16; 3],
    min_brightness: f32,
}

impl Default for Palette {
    fn default() -> Self {
        Self::from_state([30000, 15000, 35000], 0.5)
    }
}

impl Palette {
    /// Create a new [`Palette`] with the given pre-chosen state.
    ///
    /// The minimum brightness can be used to control the colour brightness (0.0 - 1.0). The default is 0.5.
    pub fn from_state(state: [u16; 3], min_brightness: f32) -> Self {
        Self { state, min_brightness: min_brightness.max(0.0).min(1.0) }
    }

    /// Create a new [`Palette`] with the default state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate the next colour in the sequence.
    pub fn random(&mut self) -> Color {
        for i in 0..3 {
            // magic constant, one of only two that have this property!
            self.state[i] = (self.state[i] as usize).wrapping_add(40503 * (i * 4 + 1130)) as u16;
        }
        Color::Fixed(
            16 + ((self.state[2] as f32 / 65535.0 * (1.0 - self.min_brightness) + self.min_brightness) * 5.0
                + (self.state[1] as f32 / 65535.0 * (1.0 - self.min_brightness) + self.min_brightness) * 30.0
                + (self.state[0] as f32 / 65535.0 * (1.0 - self.min_brightness) + self.min_brightness) * 180.0)
                as u8,
        )
    }
}
