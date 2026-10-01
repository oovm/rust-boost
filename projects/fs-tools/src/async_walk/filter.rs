/// Action to take for a discovered path in [`super::Walker`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Filter {
    /// Skip the current entry.
    Ignore,
    /// Skip the current entry and do not traverse its children.
    IgnoreDir,
    /// Keep the entry and continue traversal.
    Continue,
}
