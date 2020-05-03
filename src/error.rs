use std::fmt;
use thiserror::Error;

use crate::Op;

#[derive(Error, PartialEq, Eq, Clone, Debug)]
pub enum ChronofoldError<A: fmt::Debug + fmt::Display, T: fmt::Debug> {
    #[error("unknown reference {}", (.0).reference.as_ref().expect("reference must not be `None`"))]
    UnknownReference(Op<A, T>),
    #[error("existing timestamp {}", (.0).id)]
    ExistingTimestamp(Op<A, T>),
}
