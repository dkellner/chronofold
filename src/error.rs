use std::fmt;

use crate::{Change, Op, Timestamp};

/// Represents errors that can occur when applying an op.
///
/// Note that this implements `Debug`, `Display` and `Error` for all types `T`,
/// as the contents of changes are omitted from any output.
#[derive(PartialEq, Eq, Clone)]
pub enum ChronofoldError<A, T> {
    UnknownReference(Op<A, T>),
    ExistingTimestamp(Op<A, T>),
}

impl<A: fmt::Debug + fmt::Display + Copy, T> fmt::Debug for ChronofoldError<A, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ChronofoldError::*;
        let (name, op) = match self {
            UnknownReference(op) => ("UnknownReference", op),
            ExistingTimestamp(op) => ("ExistingTimestamp", op),
        };
        f.debug_tuple(name).field(&DebugOp::from(op)).finish()
    }
}

impl<A: fmt::Debug + fmt::Display + Copy, T> fmt::Display for ChronofoldError<A, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ChronofoldError::*;
        match self {
            UnknownReference(op) => write!(
                f,
                "unknown reference {}",
                op.reference.as_ref().expect("reference must not be `None`")
            ),
            ExistingTimestamp(op) => write!(f, "existing timestamp {}", op.id),
        }
    }
}

impl<A: fmt::Debug + fmt::Display + Copy, T> std::error::Error for ChronofoldError<A, T> {}

#[derive(Debug)]
struct DebugOp<A> {
    id: Timestamp<A>,
    reference: Option<Timestamp<A>>,
    change: Change<Omitted>,
}

impl<A: fmt::Debug + fmt::Display + Copy, T> From<&Op<A, T>> for DebugOp<A> {
    fn from(source: &Op<A, T>) -> Self {
        use Change::*;
        Self {
            id: source.id,
            reference: source.reference,
            change: match source.change {
                Insert(_) => Insert(Omitted),
                Delete => Delete,
            },
        }
    }
}

#[derive(Debug)]
struct Omitted;
