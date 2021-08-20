use std::fmt;
use std::ops::{Add, Index, Sub};

use crate::offsetmap::Offset;
use crate::{Author, Change, Chronofold};

/// An index in the log of the chronofold.
///
/// The indices are `usize` as they are used to index into `Vec`s.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LogIndex(pub usize);

impl<A: Author, T> Index<LogIndex> for Chronofold<A, T> {
    type Output = Change<T>;

    fn index(&self, index: LogIndex) -> &Self::Output {
        &self.log[index.0].0
    }
}

impl fmt::Display for LogIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<A: Author, T> Chronofold<A, T> {
    /// Returns the index of the last log entry (in log order).
    pub fn last_index(&self) -> Option<LogIndex> {
        if !self.log.is_empty() {
            Some(LogIndex(self.log.len() - 1))
        } else {
            None
        }
    }

    /// Returns the previous log index (causal order).
    ///
    /// Unlike `index`, this function never panics. It returns `None` in two
    /// cases:
    ///   1. `index` is the first index (causal order).
    ///   2. `index` is out of bounds.
    pub(crate) fn index_before(&self, index: LogIndex) -> Option<LogIndex> {
        if matches!(self.log.get(index.0).map(|e| &e.0), Some(Change::Root)) {
            Some(index)
        } else if let Some(reference) = self.references.get(&index) {
            self.iter_log_indices_causal_range(reference..index)
                .map(|(_, idx, _)| idx)
                .last()
        } else {
            None
        }
    }

    /// Returns the next log index (causal order).
    ///
    /// Unlike `index`, this function never panics. It returns `None` in two
    /// cases:
    ///   1. `index` is the last index (causal order).
    ///   2. `index` is out of bounds.
    pub(crate) fn index_after(&self, index: LogIndex) -> Option<LogIndex> {
        self.next_indices.get(&index)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct RelativeNextIndex(pub isize);

impl Default for RelativeNextIndex {
    fn default() -> Self {
        RelativeNextIndex(1)
    }
}

impl Offset<LogIndex> for RelativeNextIndex {
    fn add(&self, value: &LogIndex) -> LogIndex {
        LogIndex((value.0 as isize + self.0) as usize)
    }

    fn sub(a: &LogIndex, b: &LogIndex) -> Self {
        RelativeNextIndex(a.0 as isize - b.0 as isize)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct RelativeReference(pub isize);

impl Default for RelativeReference {
    fn default() -> Self {
        RelativeReference(-1)
    }
}

impl Offset<LogIndex> for RelativeReference {
    fn add(&self, value: &LogIndex) -> LogIndex {
        LogIndex((value.0 as isize + self.0) as usize)
    }

    fn sub(a: &LogIndex, b: &LogIndex) -> Self {
        RelativeReference(a.0 as isize - b.0 as isize)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct IndexShift(pub usize);

impl Add<&IndexShift> for &LogIndex {
    type Output = LogIndex;

    fn add(self, other: &IndexShift) -> LogIndex {
        LogIndex(self.0 + other.0)
    }
}

impl Sub<&IndexShift> for &LogIndex {
    type Output = LogIndex;

    fn sub(self, other: &IndexShift) -> LogIndex {
        LogIndex(self.0 - other.0)
    }
}

// TODO: Does it make sense to introduce a `Position` type for indexing into
// the chronofold? This would be slower as we have to access the nth element of
// the linked list. If we do so, we should return `(LogIndex, T)` to allow
// editing of the accessed value.
