use std::fmt;
use thiserror::Error;

use crate::Timestamp;

#[derive(Error, Clone, Debug)]
pub enum ChronofoldError<A: fmt::Display + fmt::Debug> {
    #[error("unknown timestamp {0}")]
    UnknownTimestamp(Timestamp<A>),
}
