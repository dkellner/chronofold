use std::fmt;
use thiserror::Error;

use crate::Timestamp;

#[derive(Error, PartialEq, Eq, Clone, Debug)]
pub enum ChronofoldError<A: fmt::Display + fmt::Debug> {
    #[error("unknown timestamp {0}")]
    UnknownTimestamp(Timestamp<A>),
    #[error("existing timestamp {0}")]
    ExistingTimestamp(Timestamp<A>),
}
