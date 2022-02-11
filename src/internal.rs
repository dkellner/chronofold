use crate::index::{IndexShift, RelativeNextIndex};
use crate::offsetmap::Offset;
use crate::{Author, Change, Chronofold, LogIndex, Timestamp};

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
        match (reference, change) {
            (None, Change::Root) => None,
            (None, Change::Insert(_)) => None,
            (_, Change::Root) => {
                // Roots cannot reference other entries.
                // XXX: Should we cover this by the type system?
                unreachable!()
            }
            (Some(reference), _change) => {
                if let Some((_, idx, _)) = self
                    .iter_log_indices_causal_range(reference..)
                    .filter(|(_, i, _)| {
                        self.references.get(i) == Some(reference)
                            && self.timestamp(*i).unwrap() > id
                    })
                    .last()
                {
                    self.iter_subtree(idx).last()
                } else {
                    Some(reference)
                }
            }
            (None, _change) => {
                // Non-roots have to reference another entry.
                // XXX: Should we cover this by the type system?
                unreachable!()
            }
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
            // Inserting another root will result in two disjunct subsequences.
            next_index = None;
        }

        if let (Change::Delete, Some(deleted)) = (&change, reference) {
            self.mark_as_deleted(deleted, new_index);
        }

        // Append to the chronofold's log and secondary logs.
        self.log.push((change, None));
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
        reference: LogIndex,
        changes: I,
    ) -> Option<LogIndex>
    where
        I: IntoIterator<Item = Change<T>>,
    {
        let mut last_id = None;
        let mut last_next_index = None;

        let mut predecessor = reference;

        let mut changes = changes.into_iter();
        if let Some(first_change) = changes.next() {
            let new_index = LogIndex(self.log.len());
            let id = Timestamp(new_index, author);
            last_id = Some(id);

            // Set the predecessors next index to our new change's index while
            // keeping it's previous next index for ourselves.
            last_next_index = Some(self.next_indices.get(&predecessor));
            self.next_indices.set(predecessor, Some(new_index));

            if let Change::Delete = &first_change {
                self.mark_as_deleted(predecessor, new_index);
            }

            self.log.push((first_change, None));
            self.authors.set(new_index, author);
            self.index_shifts.set(new_index, IndexShift(0));
            self.references.set(new_index, Some(predecessor));

            predecessor = new_index;
        }

        for change in changes {
            let new_index = RelativeNextIndex::default().add(&predecessor);
            let id = Timestamp(new_index, author);
            last_id = Some(id);

            if let Change::Delete = &change {
                self.mark_as_deleted(predecessor, new_index);
            }

            // Append to the chronofold's log and secondary logs.
            self.log.push((change, None));

            predecessor = new_index;
        }

        if let (Some(id), Some(next_index)) = (last_id, last_next_index) {
            self.next_indices.set(id.0, next_index);
            self.version.inc(&id);
            Some(id.0)
        } else {
            None
        }
    }

    fn mark_as_deleted(&mut self, index: LogIndex, deletion: LogIndex) {
        self.log[index.0].1 = Some(match self.log[index.0].1 {
            None => deletion,
            Some(other_deletion) => LogIndex(usize::min(deletion.0, other_deletion.0)),
        })
    }
}
