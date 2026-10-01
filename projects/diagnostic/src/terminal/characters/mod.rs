/// Possible character sets to use when rendering diagnostics.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BuiltinDrawer {
    /// Unicode characters (an attempt is made to use only commonly-supported characters).
    Unicode,
    /// ASCII-only characters.
    Ascii,
}

/// A trait for types that can be used to draw diagnostics.
pub trait Draw {
    /// Get the character set to use when rendering diagnostics.
    fn get_elements(&self) -> DrawElements;
}

impl Draw for BuiltinDrawer {
    fn get_elements(&self) -> DrawElements {
        match self {
            BuiltinDrawer::Unicode => DrawElements {
                hbar: '─',
                vbar: '│',
                xbar: '┼',
                vbar_break: '┆',
                vbar_gap: '┆',
                uarrow: '🭯',
                rarrow: '▶',
                ltop: '╭',
                mtop: '┬',
                rtop: '╮',
                lbot: '╰',
                mbot: '┴',
                rbot: '╯',
                lbox: '[',
                rbox: ']',
                lcross: '├',
                rcross: '┤',
                underbar: '┬',
                underline: '─',
            },
            BuiltinDrawer::Ascii => DrawElements {
                hbar: '-',
                vbar: '|',
                xbar: '+',
                vbar_break: '*',
                vbar_gap: ':',
                uarrow: '^',
                rarrow: '>',
                ltop: ',',
                mtop: 'v',
                rtop: '.',
                lbot: '`',
                mbot: '^',
                rbot: '\'',
                lbox: '[',
                rbox: ']',
                lcross: '|',
                rcross: '|',
                underbar: '|',
                underline: '^',
            },
        }
    }
}

/// The character set used by formatter
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DrawElements {
    /// Horizontal bar, eg: `─, -`
    pub hbar: char,
    /// Vertical bar, eg: `│, |`
    pub vbar: char,
    /// Cross bar, eg: `┼, +`
    pub xbar: char,
    /// Vertical bar break, eg: `┆, *`
    pub vbar_break: char,
    /// Vertical bar gap, eg: `┆, :`
    pub vbar_gap: char,
    /// Up arrow, eg: `🭯, ^`
    pub uarrow: char,
    /// Right arrow, eg: `▶, >`
    pub rarrow: char,
    /// Left top corner, eg: `╭, ,`
    pub ltop: char,
    /// Middle top, eg: `┬, v`
    pub mtop: char,
    /// Right top corner, eg: `╮, .`
    pub rtop: char,
    /// Left bottom corner, eg: `╰, ``
    pub lbot: char,
    /// Middle bottom, eg: `┴, ^`
    pub rbot: char,
    /// Right bottom corner, eg: `╯, '`
    pub mbot: char,
    /// Left box, eg: `[`
    pub lbox: char,
    /// Right box, eg: `]`
    pub rbox: char,
    /// Left cross, eg: `├, |`
    pub lcross: char,
    /// Right cross, eg: `┤, |`
    pub rcross: char,
    /// Under bar, eg: `┬, |`
    pub underbar: char,
    /// Underline, eg: `─, ^`
    pub underline: char,
}
