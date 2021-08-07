use std::ops::{Bound, RangeBounds};

use crate::{Author, Change, Chronofold, FromLocalValue, LogIndex, Op, Timestamp};

/// An editing session tied to one author.
///
/// `Session` provides a lot of functions you might know from `Vec` or
/// `VecDeque`. Under the hood, `Session` will append changes to the
/// chronofolds log.
///
/// Note that `Session` has a mutable (exclusive) borrow of a chronofold. So
/// Rust's ownership rules enforce that there is always just one `Session` per
/// chronofold.
#[derive(Debug)]
pub struct Session<'a, A, T> {
    chronofold: &'a mut Chronofold<A, T>,
    author: A,
    first_index: LogIndex,
}

impl<'a, A: Author, T> Session<'a, A, T> {
    /// Creates an editing session for a single author.
    pub fn new(author: A, chronofold: &'a mut Chronofold<A, T>) -> Self {
        let first_index = chronofold.next_log_index();
        Self {
            chronofold,
            author,
            first_index,
        }
    }

    /// Clears the chronofold, removing all elements.
    pub fn clear(&mut self) {
        let indices = self
            .chronofold
            .iter()
            .map(|(_, idx)| idx)
            .collect::<Vec<_>>();
        for idx in indices {
            self.remove(idx);
        }
    }

    /// Appends an element to the back of the chronofold and returns the new
    /// element's log index.
    pub fn push_back(&mut self, value: T) -> LogIndex {
        if let Some((_, last_index)) = self.chronofold.iter().last() {
            self.insert_after(last_index, value)
        } else {
            // no non-deleted entries left
            self.insert_after(self.as_ref().root, value)
        }
    }

    /// Prepends an element to the chronofold and returns the new element's log
    /// index.
    pub fn push_front(&mut self, value: T) -> LogIndex {
        self.insert_after(self.as_ref().root, value)
    }

    /// Inserts an element after the element with log index `index` and returns
    /// the new element's log index.
    ///
    /// If `index == None`, the element will be inserted at the beginning.
    pub fn insert_after(&mut self, index: LogIndex, value: T) -> LogIndex {
        self.apply_change(index, Change::Insert(value))
    }

    /// Removes the element with log index `index` from the chronofold.
    ///
    /// Note that this just marks the element as deleted, not actually modify
    /// the log apart from appending a `Change::Delete`.
    pub fn remove(&mut self, index: LogIndex) {
        self.apply_change(index, Change::Delete);
    }

    /// Extends the chronofold with the contents of `iter`, returns the log
    /// index of the last inserted element, if any.
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) -> Option<LogIndex> {
        let oob = LogIndex(self.chronofold.log.len());
        self.splice(oob..oob, iter)
    }

    #[allow(clippy::needless_collect)] // collect is needed due to borrowing
    /// Replaces the specified range in the chronofold with the given
    /// `replace_with` iterator and returns the log index of the last inserted
    /// element, if any.
    pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> Option<LogIndex>
    where
        I: IntoIterator<Item = T>,
        R: RangeBounds<LogIndex>,
    {
        let last_idx = match range.start_bound() {
            Bound::Unbounded => None,
            Bound::Included(idx) => self.chronofold.index_before(*idx),
            Bound::Excluded(idx) => Some(*idx),
        }
        .unwrap_or(self.as_ref().root);
        let to_remove: Vec<LogIndex> = self
            .chronofold
            .iter_range(range)
            .map(|(_, idx)| idx)
            .collect();
        for idx in to_remove.into_iter() {
            self.remove(idx);
        }
        self.apply_changes(last_idx, replace_with.into_iter().map(Change::Insert))
    }

    pub fn create_root(&mut self) -> LogIndex {
        let new_index = LogIndex(self.chronofold.log.len());
        self.chronofold
            .apply_change(Timestamp(new_index, self.author), None, Change::Root)
    }

    fn apply_change(&mut self, reference: LogIndex, change: Change<T>) -> LogIndex {
        self.apply_changes(reference, Some(change)).unwrap()
    }

    fn apply_changes<I>(&mut self, reference: LogIndex, changes: I) -> Option<LogIndex>
    where
        I: IntoIterator<Item = Change<T>>,
    {
        self.chronofold
            .apply_local_changes(self.author, reference, changes)
    }

    /// Returns an iterator over ops in log order, that where created in this
    /// session.
    pub fn iter_ops<V>(&'a self) -> impl Iterator<Item = Op<A, V>> + 'a
    where
        V: FromLocalValue<'a, A, T> + 'a,
    {
        self.chronofold
            .iter_ops(self.first_index..)
            .filter(move |op| op.id.1 == self.author)
    }
}

impl<A: Author, T> AsRef<Chronofold<A, T>> for Session<'_, A, T> {
    fn as_ref(&self) -> &Chronofold<A, T> {
        self.chronofold
    }
}

impl<A: Author, T> AsMut<Chronofold<A, T>> for Session<'_, A, T> {
    fn as_mut(&mut self) -> &mut Chronofold<A, T> {
        self.chronofold
    }
}
