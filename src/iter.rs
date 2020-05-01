use std::collections::HashSet;
use std::ops::{Bound, RangeBounds};

use crate::{Author, Change, Chronofold, LogIndex, Op};

impl<A: Author, T> Chronofold<A, T> {
    /// Returns an iterator over the log indices in causal order.
    ///
    /// TODO: The name is a bit unwieldy. I'm reluctant to add it to the public
    /// API before giving it more thought.
    pub(crate) fn iter_log_indices_causal_range<'a, R>(
        &'a self,
        range: R,
    ) -> impl Iterator<Item = LogIndex> + 'a
    where
        R: RangeBounds<LogIndex>,
    {
        let mut current = match range.start_bound() {
            Bound::Unbounded => self.root,
            Bound::Included(idx) => Some(*idx),
            Bound::Excluded(idx) => self.index_after(*idx),
        };
        let first_excluded = match range.end_bound() {
            Bound::Unbounded => None,
            Bound::Included(idx) => self.index_after(*idx),
            Bound::Excluded(idx) => Some(*idx),
        };
        std::iter::from_fn(move || {
            let idx = current?;
            current = self.index_after(idx);
            if Some(idx) != first_excluded {
                Some(idx)
            } else {
                None
            }
        })
    }

    /// Returns an iterator over a subtree.
    ///
    /// The first item is always `root`.
    pub(crate) fn iter_subtree<'a>(
        &'a self,
        root: LogIndex,
    ) -> impl Iterator<Item = LogIndex> + 'a {
        let mut subtree: HashSet<LogIndex> = HashSet::new();
        self.iter_log_indices_causal_range(root..)
            .filter_map(move |idx| {
                if idx == root || subtree.contains(&self.references[idx.0]?) {
                    subtree.insert(idx);
                    Some(idx)
                } else {
                    None
                }
            })
    }

    /// Returns an iterator over elements and their log indices in causal order.
    pub fn iter(&self) -> impl Iterator<Item = (&T, LogIndex)> {
        self.iter_range(..)
    }

    /// Returns an iterator over elements and their log indices in causal order.
    pub fn iter_range<R>(&self, range: R) -> impl Iterator<Item = (&T, LogIndex)>
    where
        R: RangeBounds<LogIndex>,
    {
        self.iter_log_indices_causal_range(range)
            .filter_map(move |i| match (&self.log[i.0], self.deleted[i.0]) {
                (Change::Insert(value), false) => Some((value, i)),
                _ => None,
            })
    }

    /// Returns an iterator over elements in causal order.
    pub fn iter_elements(&self) -> impl Iterator<Item = &T> {
        self.iter().map(|(v, _)| v)
    }

    /// Returns an iterator over changes in log order.
    pub fn iter_changes(&self) -> impl Iterator<Item = &Change<T>> {
        self.log.iter()
    }
}

impl<A: Author, T> Chronofold<A, T>
where
    T: Clone,
{
    /// Returns an iterator over ops in log order.
    pub fn iter_ops<'a>(&'a self) -> impl Iterator<Item = Op<A, T>> + 'a {
        self.log
            .iter()
            .cloned()
            .enumerate()
            .map(move |(i, change)| {
                Op::new(
                    self.timestamps[i],
                    self.references[i].map(|r| self.timestamps[r.0]),
                    change,
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter_subtree() {
        let mut cfold = Chronofold::<u8, char>::default();
        cfold.session(1).extend("013".chars());
        cfold.session(1).insert_after(Some(LogIndex(1)), '2');
        assert_eq!(
            vec![LogIndex(1), LogIndex(3), LogIndex(2)],
            cfold.iter_subtree(LogIndex(1)).collect::<Vec<_>>()
        );
    }
}
