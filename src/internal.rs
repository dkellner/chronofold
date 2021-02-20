use crate::index::{IndexShift, RelativeNextIndex};
use crate::offsetmap::Offset;
use crate::{Author, Change, Chronofold, LogIndex, Timestamp};

use std::matches;

impl<A: Author, T> Chronofold<A, T> {
    pub(crate) fn next_log_index(&self) -> LogIndex {
        LogIndex(self.log.len())
    }

    pub(crate) fn find_predecessor(
        &self,
        id: Timestamp<A>,
        reference: Option<LogIndex>,
        change: &Change<T>,
    ) -> Option<LogIndex> {
        match change {
            Change::Delete => reference, // deletes have priority
            _ => {
                if let Some((_, idx)) = self
                    .iter_log_indices_causal_range(..) // TODO: performance
                    .filter(|(_, i)| self.references.get(i) == reference)
                    .filter(|(c, i)| matches!(c, Change::Delete) || self.timestamp(i).unwrap() > id)
                    .last()
                {
                    self.iter_subtree(idx).last()
                } else {
                    reference
                }
            }
        }
    }

    pub(crate) fn log_index(&self, timestamp: &Timestamp<A>) -> Option<LogIndex> {
        for i in (timestamp.0).0..self.log.len() {
            if self.timestamp(&LogIndex(i)).unwrap() == *timestamp {
                return Some(LogIndex(i));
            }
        }
        None
    }

    pub(crate) fn timestamp(&self, index: &LogIndex) -> Option<Timestamp<A>> {
        if let (Some(shift), Some(author)) = (self.index_shifts.get(index), self.authors.get(index))
        {
            Some(Timestamp(index - shift, *author))
        } else {
            None
        }
    }

    pub(crate) fn apply_change(
        &mut self,
        id: Timestamp<A>,
        reference: Option<LogIndex>,
        change: Change<T>,
    ) -> LogIndex {
        // Find the predecessor to `op`.
        let predecessor = self.find_predecessor(id, reference, &change);

        // Set the predecessors next index to our new change's index while
        // keeping it's previous next index for ourselves.
        let new_index = LogIndex(self.log.len());
        let next_index;
        if let Some(idx) = predecessor {
            next_index = self.next_indices.get(&idx);
            self.next_indices.set(idx, Some(new_index));
        } else {
            next_index = self.root;
            self.root = Some(new_index);
        }

        // Append to the chronofold's log and secondary logs.
        self.log.push(change);
        self.next_indices.set(new_index, next_index);
        self.authors.set(new_index, id.1);
        self.index_shifts
            .set(new_index, IndexShift(new_index.0 - (id.0).0));
        self.references.set(new_index, reference);

        // Increment version.
        self.version.inc(&id);

        new_index
    }

    /// Applies consecutive local changes.
    ///
    /// For local changes the following optimizations can be applied:
    /// - id equals (log index, author)
    /// - predecessor always equals reference (no preemptive siblings)
    /// - next index has to be set only for the first and the last change
    pub(crate) fn apply_local_changes<I>(
        &mut self,
        author: A,
        reference: Option<LogIndex>,
        changes: I,
    ) -> Option<LogIndex>
    where
        I: IntoIterator<Item = Change<T>>,
    {
        let mut last_id = None;
        let mut last_next_index = None;

        let mut predecessor = reference.map(|r| match self.find_last_delete(r) {
            Some(idx) => idx,
            None => r,
        });

        let mut changes = changes.into_iter();
        if let Some(first_change) = changes.next() {
            let new_index = LogIndex(self.log.len());
            let id = Timestamp(new_index, author);
            last_id = Some(id);

            // Set the predecessors next index to our new change's index while
            // keeping it's previous next index for ourselves.
            if let Some(idx) = predecessor {
                last_next_index = Some(self.next_indices.get(&idx));
                self.next_indices.set(idx, Some(new_index));
            } else {
                last_next_index = Some(self.root);
                self.root = Some(new_index);
            }

            self.log.push(first_change);
            self.authors.set(new_index, author);
            self.index_shifts.set(new_index, IndexShift(0));
            self.references.set(new_index, predecessor);

            predecessor = Some(new_index);
        }

        for change in changes {
            let new_index = RelativeNextIndex::default().add(predecessor.as_ref().unwrap());
            let id = Timestamp(new_index, author);
            last_id = Some(id);

            // Append to the chronofold's log and secondary logs.
            self.log.push(change);

            predecessor = Some(new_index);
        }

        if let (Some(id), Some(next_index)) = (last_id, last_next_index) {
            self.next_indices.set(id.0, next_index);
            self.version.inc(&id);
            Some(id.0)
        } else {
            None
        }
    }

    pub(crate) fn find_last_delete(&self, reference: LogIndex) -> Option<LogIndex> {
        self.iter_log_indices_causal_range(reference..)
            .skip(1)
            .filter(|(c, idx)| {
                matches!(c, Change::Delete) && self.references.get(idx) == Some(reference)
            })
            .last()
            .map(|(_, idx)| idx)
    }
}
