/// An entry in the chronofold's log.
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(untagged))]
pub enum Change<T> {
    Insert(T),
    Delete,
}

impl<T> Change<T> {
    /// Converts from `&Change<T>` to `Change<&T>`.
    pub fn as_ref(&self) -> Change<&T> {
        use Change::*;
        match *self {
            Insert(ref x) => Insert(x),
            Delete => Delete,
        }
    }
}

impl<T: Clone> Change<&T> {
    /// Maps a Change<&T> to a Change<T> by cloning its contents.
    pub fn cloned(self) -> Change<T> {
        use Change::*;
        match self {
            Insert(x) => Insert(x.clone()),
            Delete => Delete,
        }
    }
}
