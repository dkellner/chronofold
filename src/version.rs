use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;

use crate::{Author, Chronofold, LogIndex, Op, Timestamp};

/// A vector clock representing the chronofold's version.
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Version<A: Author> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    log_indices: BTreeMap<A, LogIndex>,
}

impl<A: Author> Version<A> {
    /// Constructs a new, empty version.
    pub fn new() -> Self {
        Self::default()
    }

    /// Increments the version using a timestamp.
    pub fn inc(&mut self, timestamp: &Timestamp<A>) {
        self.log_indices
            .entry(timestamp.1)
            .and_modify(|t| *t = LogIndex(usize::max(t.0, (timestamp.0).0)))
            .or_insert(timestamp.0);
    }

    /// Returns an iterator over the timestamps in this version.
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Timestamp<A>> + 'a {
        self.log_indices.iter().map(|(a, i)| Timestamp(*i, *a))
    }
}

impl<A: Author> Default for Version<A> {
    fn default() -> Self {
        Self {
            log_indices: BTreeMap::new(),
        }
    }
}

impl<A: Author> PartialOrd for Version<A> {
    fn partial_cmp(&self, other: &Version<A>) -> Option<Ordering> {
        let gt = |lhs: &Version<A>, rhs: &Version<A>| {
            rhs.log_indices.iter().all(|(a, rhs_idx)| {
                lhs.get(a)
                    .map(|lhs_idx| lhs_idx >= *rhs_idx)
                    .unwrap_or(false)
            })
        };

        if self == other {
            Some(Ordering::Equal)
        } else if gt(self, other) {
            Some(Ordering::Greater)
        } else if gt(other, self) {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}

impl<A: Author> Version<A> {
    /// Returns the version's log index for `author`.
    pub fn get(&self, author: &A) -> Option<LogIndex> {
        self.log_indices.get(author).cloned()
    }
}

impl<A: Author, T: Clone + fmt::Debug> Chronofold<A, T> {
    /// Returns a vector clock representing the version of this chronofold.
    pub fn version(&self) -> &Version<A> {
        &self.version
    }

    /// Returns an iterator over ops newer than the given version in log order.
    pub fn iter_newer_ops<'a>(
        &'a self,
        version: &'a Version<A>,
    ) -> impl Iterator<Item = Op<A, T>> + 'a {
        // TODO: Don't iterate over all ops in cases where that is not
        // necessary.
        self.iter_ops(..)
            .filter(move |op| match version.log_indices.get(&op.id.1) {
                None => true,
                Some(idx) => op.id.0 > *idx,
            })
    }
}
