use thiserror::Error;

use crate::{Author, Timestamp};

#[derive(Error, Clone, Debug)]
pub enum ChronofoldError<A: Author> {
    #[error("unknown timestamp {0}")]
    UnknownTimestamp(Timestamp<A>),
}
