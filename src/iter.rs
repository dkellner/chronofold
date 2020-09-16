use std::collections::HashSet;
use std::fmt;
use std::matches;
use std::ops::{Bound, RangeBounds};

use crate::{Author, Change, Chronofold, LogIndex, Op};

impl<A: Author, T> Chronofold<A, T> {
    /// Returns an iterator over the log indices in causal order.
    ///
    /// TODO: The name is a bit unwieldy. I'm reluctant to add it to the public
    /// API before giving it more thought.
    pub(crate) fn iter_log_indices_causal_range<'a, R>(&'a self, range: R) -> CausalIter<'a, A, T>
    where
        R: RangeBounds<LogIndex>,
    {
        let current = match range.start_bound() {
            Bound::Unbounded => self.root,
            Bound::Included(idx) => Some(*idx),
            Bound::Excluded(idx) => self.index_after(*idx),
        };
        let first_excluded = match range.end_bound() {
            Bound::Unbounded => None,
            Bound::Included(idx) => self.index_after(*idx),
            Bound::Excluded(idx) => Some(*idx),
        };
        CausalIter {
            cfold: self,
            current,
            first_excluded,
        }
    }

    /// Returns an iterator over a subtree.
    ///
    /// The first item is always `root`.
    pub(crate) fn iter_subtree<'a>(
        &'a self,
        root: LogIndex,
    ) -> impl Iterator<Item = LogIndex> + 'a {
        let mut subtree: HashSet<LogIndex> = HashSet::new();
        self.iter_log_indices_causal_range(..)
            .filter_map(move |(_, idx)| {
                if idx == root || subtree.contains(&self.references.get(&idx)?) {
                    subtree.insert(idx);
                    Some(idx)
                } else {
                    None
                }
            })
    }

    /// Returns an iterator over elements and their log indices in causal order.
    pub fn iter(&self) -> Iter<A, T> {
        self.iter_range(..)
    }

    /// Returns an iterator over elements and their log indices in causal order.
    pub fn iter_range<R>(&self, range: R) -> Iter<A, T>
    where
        R: RangeBounds<LogIndex>,
    {
        let mut causal_iter = self.iter_log_indices_causal_range(range);
        let current = causal_iter.next();
        Iter {
            causal_iter,
            current,
        }
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

impl<A: Author, T: Clone + fmt::Debug> Chronofold<A, T> {
    /// Returns an iterator over ops in log order.
    pub fn iter_ops<'a, R>(&'a self, range: R) -> impl Iterator<Item = Op<A, T>> + 'a
    where
        R: RangeBounds<LogIndex> + 'a,
    {
        self.log
            .iter()
            .cloned()
            .enumerate()
            .filter(move |(i, _)| range.contains(&LogIndex(*i)))
            .map(move |(i, change)| {
                Op::new(
                    self.timestamp(&LogIndex(i)).unwrap(),
                    self.references
                        .get(&LogIndex(i))
                        .map(|r| self.timestamp(&r).unwrap()),
                    change,
                )
            })
    }
}

pub(crate) struct CausalIter<'a, A: Author, T> {
    cfold: &'a Chronofold<A, T>,
    current: Option<LogIndex>,
    first_excluded: Option<LogIndex>,
}

impl<'a, A: Author, T> Iterator for CausalIter<'a, A, T> {
    type Item = (&'a Change<T>, LogIndex);

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.take() {
            Some(current) if Some(current) != self.first_excluded => {
                self.current = self.cfold.index_after(current);
                Some((&self.cfold.log[current.0], current))
            }
            _ => None,
        }
    }
}

pub struct Iter<'a, A: Author, T> {
    causal_iter: CausalIter<'a, A, T>,
    current: Option<(&'a Change<T>, LogIndex)>,
}

impl<'a, A: Author, T> Iterator for Iter<'a, A, T> {
    type Item = (&'a T, LogIndex);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (skipped, next) =
                skip_while(&mut self.causal_iter, |(c, _)| matches!(c, Change::Delete));
            if skipped == 0 {
                // the current item is not deleted
                match self.current.take() {
                    None => {
                        return None;
                    }
                    Some((Change::Insert(v), idx)) => {
                        self.current = next;
                        return Some((v, idx));
                    }
                    _ => unreachable!(),
                }
            } else {
                // the current item is deleted
                self.current = next;
            }
        }
    }
}

/// Skips items where `predicate` returns true.
///
/// Note that while this works like `Iterator::skip_while`, it does not create
/// a new iterator. Instead `iter` is modified.
fn skip_while<I, P>(iter: &mut I, predicate: P) -> (usize, Option<I::Item>)
where
    I: Iterator,
    P: Fn(&I::Item) -> bool,
{
    let mut skipped = 0;
    loop {
        match iter.next() {
            Some(item) if !predicate(&item) => {
                return (skipped, Some(item));
            }
            None => {
                return (skipped, None);
            }
            _ => skipped += 1,
        }
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

    #[test]
    fn skip_while() {
        let mut iter = 2..10;
        let result = super::skip_while(&mut iter, |i| i < &7);
        assert_eq!((5, Some(7)), result);
        assert_eq!(vec![8, 9], iter.collect::<Vec<_>>());

        let mut iter2 = 2..10;
        let result = super::skip_while(&mut iter2, |i| i < &20);
        assert_eq!((8, None), result);
    }
}
