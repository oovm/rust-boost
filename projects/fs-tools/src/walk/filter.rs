/// Action taken for an entry discovered during a walk.
///
/// A walker applies this decision before it yields the entry, so a folder that is rejected with
/// [`Filter::IgnoreFolder`] is never traversed. Filtering after the entry was yielded cannot prune
/// the tree, which is the reason this decision exists.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Filter {
    /// Yield the entry, and traverse its children when it is a folder.
    #[default]
    Continue,
    /// Do not yield the entry, but still traverse its children when it is a folder.
    Ignore,
    /// Do not yield the entry, and do not traverse its children.
    IgnoreFolder,
}
