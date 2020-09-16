//! # Chronofold
//!
//! Chronofold is a conflict-free replicated data structure (a.k.a. *CRDT*) for
//! versioned text.
//!
//! This crate aims to offer a fast implementation with an easy-to-use
//! `Vec`-like API. It should be near impossible to shoot yourself in the foot
//! and end up with corrupted or lost data.
//!
//! **Note:** We are not there yet! While this implementation should be
//! correct, it is not yet optimized for speed and memory usage. The API might
//! see some changes as we continue to explore different use cases.
//!
//! This implementation is based on ideas published in the paper ["Chronofold:
//! a data structure for versioned text"][paper] by Victor Grishchenko and
//! Mikhail Patrakeev. If you look for a formal introduction to what a
//! chronofold is, reading that excellent paper is highly recommended!
//!
//! [paper]: https://arxiv.org/abs/2002.09511
//!
//! # Example usage
//!
//! ```rust
//! use chronofold::{Chronofold, LogIndex, Op};
//!
//! type AuthorId = &'static str;
//!
//! // Alice creates a chronofold on her machine, makes some initial changes
//! // and sends a copy to Bob.
//! let mut cfold_a = Chronofold::<AuthorId, char>::default();
//! cfold_a.session("alice").extend("Hello chronfold!".chars());
//! let mut cfold_b = cfold_a.clone();
//!
//! // Alice adds some more text, ...
//! let ops_a: Vec<Op<AuthorId, char>> = {
//!     let mut session = cfold_a.session("alice");
//!     session.splice(
//!         LogIndex(15)..LogIndex(15),
//!         " - a data structure for versioned text".chars(),
//!     );
//!     session.iter_ops().collect()
//! };
//!
//! // ... while Bob fixes a typo.
//! let ops_b: Vec<Op<AuthorId, char>> = {
//!     let mut session = cfold_b.session("bob");
//!     session.insert_after(Some(LogIndex(10)), 'o');
//!     session.iter_ops().collect()
//! };
//!
//! // Now their respective states have diverged.
//! assert_eq!(
//!     "Hello chronfold - a data structure for versioned text!",
//!     format!("{}", cfold_a),
//! );
//! assert_eq!("Hello chronofold!", format!("{}", cfold_b));
//!
//! // As soon as both have seen all ops, their states have converged.
//! for op in ops_a {
//!     cfold_b.apply(op).unwrap();
//! }
//! for op in ops_b {
//!     cfold_a.apply(op).unwrap();
//! }
//! let final_text = "Hello chronofold - a data structure for versioned text!";
//! assert_eq!(final_text, format!("{}", cfold_a));
//! assert_eq!(final_text, format!("{}", cfold_b));
//! ```

// As we only have a handful of public items, we've decided to re-export
// everything in the crate root and keep our internal module structure
// private. This keeps things simple for our users and gives us more
// flexibility in restructuring the crate.
mod distributed;
mod error;
mod index;
mod iter;
mod offsetmap;
mod rangemap;
mod session;
mod version;

pub use crate::distributed::*;
pub use crate::error::*;
pub use crate::index::*;
pub use crate::iter::*;
pub use crate::session::*;
pub use crate::version::*;

use std::fmt;
use std::matches;

use crate::index::{IndexShift, RelativeNextIndex, RelativeReference};
use crate::offsetmap::{Offset, OffsetMap};
use crate::rangemap::RangeFromMap;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

/// An entry in the chronofold's log.
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(untagged))]
pub enum Change<T> {
    Insert(T),
    Delete,
}

/// A conflict-free replicated data structure for versioned sequences.
///
/// # Terminology
///
/// A chronofold can be regarded either as a log of changes or as a sequence of
/// elements. These two viewpoints require distinct terminology:
///
/// - A *log index* is a 0-based index in the log of changes. This indices are
///   stable (i.e. they stay the same after edits), but are subjective for
///   each author.
/// - An *element* is a visible (not yet deleted) value of type `T`.
/// - *Log order* refers to the chronological order in which changes were
///   added to the log. This order is subjective for each author.
/// - *Causal order* refers to the order of the linked list.
///
/// # Editing
///
/// You can edit a chronofold in two ways: Either by applying [`Op`]s, or by
/// creating a [`Session`] which has a `Vec`-like API.
///
/// # Indexing
///
/// Like [`Vec`], the `Chronofold` type allows to access values by index,
/// because it implements the [`Index`] trait. The same rules apply:
/// out-of-bound indexes cause panics, and you can use `get` to check whether
/// the index exists.
///
/// [`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
/// [`Index`]: https://doc.rust-lang.org/std/ops/trait.Index.html
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Chronofold<A: Author, T> {
    log: Vec<Change<T>>,
    root: Option<LogIndex>,
    version: Version<A>,

    next_indices: OffsetMap<LogIndex, RelativeNextIndex>,
    references: OffsetMap<LogIndex, RelativeReference>,
    authors: RangeFromMap<LogIndex, A>,
    index_shifts: RangeFromMap<LogIndex, IndexShift>,
}

impl<A: Author, T> Chronofold<A, T> {
    /// Constructs a new, empty chronofold.
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn next_log_index(&self) -> LogIndex {
        LogIndex(self.log.len())
    }

    fn find_predecessor(
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

    /// Returns `true` if the chronofold contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of elements in the chronofold.
    pub fn len(&self) -> usize {
        self.iter().count()
    }

    /// Returns a reference to a change in the chronofold's log.
    ///
    /// If `index` is out of bounds, `None` is returned.
    pub fn get(&self, index: LogIndex) -> Option<&Change<T>> {
        self.log.get(index.0)
    }

    fn log_index(&self, timestamp: &Timestamp<A>) -> Option<LogIndex> {
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
}

impl<A: Author, T: fmt::Debug> Chronofold<A, T> {
    /// Creates an editing session for a single author.
    pub fn session(&mut self, author: A) -> Session<'_, A, T> {
        Session::new(author, self)
    }

    /// Applies an op to the chronofold.
    pub fn apply(&mut self, op: Op<A, T>) -> Result<(), ChronofoldError<A, T>> {
        // Check if an op with the same id was applied already.
        // TODO: Consider adding an `apply_unchecked` variant to skip this
        // check.
        if self.log_index(&op.id).is_some() {
            return Err(ChronofoldError::ExistingTimestamp(op));
        }

        // Convert the reference timestamp, as all our internal functions work
        // with log indices.
        match op.reference {
            Some(t) => match self.log_index(&t) {
                Some(reference) => self
                    .apply_change(op.id, Some(reference), op.change)
                    .map(|_| ()),
                None => Err(ChronofoldError::UnknownReference(op)),
            },
            None => self.apply_change(op.id, None, op.change).map(|_| ()),
        }
    }

    pub(crate) fn apply_change(
        &mut self,
        id: Timestamp<A>,
        reference: Option<LogIndex>,
        change: Change<T>,
    ) -> Result<LogIndex, ChronofoldError<A, T>> {
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

        Ok(new_index)
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
    ) -> Result<Option<LogIndex>, ChronofoldError<A, T>>
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
            Ok(Some(id.0))
        } else {
            Ok(None)
        }
    }
}

impl<A: Author, T> Default for Chronofold<A, T> {
    fn default() -> Self {
        Self {
            log: Vec::default(),
            root: None,
            version: Version::default(),
            next_indices: OffsetMap::default(),
            authors: RangeFromMap::default(),
            index_shifts: RangeFromMap::default(),
            references: OffsetMap::default(),
        }
    }
}

impl<A: Author, T: fmt::Display> fmt::Display for Chronofold<A, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.iter_elements()
                .fold("".to_owned(), |s, t| s + &t.to_string())
        )
    }
}
