use std::fmt;

use crate::{Op, OpPayload};

/// Represents errors that can occur when applying an op.
///
/// Note that this implements `Debug`, `Display` and `Error` for all types `T`,
/// as the contents of changes are omitted from any output.
#[derive(PartialEq, Eq, Clone)]
pub enum ChronofoldError<A, T> {
    UnknownReference(Op<A, T>),
    ExistingTimestamp(Op<A, T>),
}

impl<A, T> fmt::Debug for ChronofoldError<A, T>
where
    A: fmt::Debug + fmt::Display + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ChronofoldError::*;
        let (name, op) = match self {
            UnknownReference(op) => ("UnknownReference", op),
            ExistingTimestamp(op) => ("ExistingTimestamp", op),
        };
        f.debug_tuple(name).field(&op.omit_value()).finish()
    }
}

impl<A, T> fmt::Display for ChronofoldError<A, T>
where
    A: fmt::Debug + fmt::Display + Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ChronofoldError::*;
        match self {
            UnknownReference(op) => write!(
                f,
                "unknown reference {}",
                op.payload
                    .reference()
                    .as_ref()
                    .expect("reference must not be `None`")
            ),
            ExistingTimestamp(op) => write!(f, "existing timestamp {}", op.id),
        }
    }
}

impl<A, T> std::error::Error for ChronofoldError<A, T> where A: fmt::Debug + fmt::Display + Copy {}

impl<A, T> Op<A, T>
where
    A: Copy,
{
    fn omit_value(&self) -> Op<A, Omitted> {
        use OpPayload::*;
        Op {
            id: self.id,
            payload: match self.payload {
                Root => Root,
                Insert(t, _) => Insert(t, Omitted),
                Delete(t) => Delete(t),
            },
        }
    }
}

#[derive(Debug)]
struct Omitted;
