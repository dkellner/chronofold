//! Distributed primitives.

use std::fmt;

use crate::{Change, LogIndex};

/// A trait alias to reduce redundancy in type declarations.
pub trait Author:
    PartialEq + Eq + PartialOrd + Ord + Clone + Copy + fmt::Debug + fmt::Display
{
}

/// Blanket implementation of `Author`.
///
/// Every type that implements the needed traits automatically implements
/// `Author` as well.
impl<T> Author for T where
    T: PartialEq + Eq + PartialOrd + Ord + Clone + Copy + fmt::Debug + fmt::Display
{
}

/// An ordered pair of the author's index and the author.
///
/// The lexicographic order of timestamps forms an arbitrary total order, that
/// is consistent with cause-effect ordering. That is, if a timestamp is
/// greater than another, its associated event either happened after the other
/// or was concurrent.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Timestamp<A>(pub LogIndex, pub A);

impl<A: fmt::Display> fmt::Display for Timestamp<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}, {}>", self.0, self.1)
    }
}

/// An operation is the unit of change in the distributed context.
///
/// Ops are independent of the subjective orders in the chronofolds'
/// logs. Different authors exchange ops to keep their local replicas
/// synchronized.
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Op<A, T> {
    pub id: Timestamp<A>,
    pub reference: Option<Timestamp<A>>, // None = root
    pub change: Change<T>,
}

impl<A, T> Op<A, T> {
    pub fn new(id: Timestamp<A>, reference: Option<Timestamp<A>>, change: Change<T>) -> Self {
        Self {
            id,
            reference,
            change,
        }
    }
}
