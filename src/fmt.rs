use crate::{Author, Chronofold};

use std::fmt;

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
