use std::fmt;

use crate::{Author, Chronofold, LogIndex};

impl<A: Author, T: fmt::Debug> Chronofold<A, T> {
    pub fn formatted_log(&self) -> String {
        let mut result = format!("{:<4} | {:<4} | {:<4} | change\n", "idx", "ref", "next");
        for (idx, change) in self.log.iter().enumerate() {
            let log_idx = LogIndex(idx);
            let formatted_ref = format_option(self.references.get(&log_idx));
            let next = format_option(self.next_indices.get(&log_idx));
            result += &format!(
                "{:<4} | {:<4} | {:<4} | {:?}\n",
                idx, formatted_ref, next, change
            );
        }
        result
    }
}

fn format_option<T: fmt::Display>(option: Option<T>) -> String {
    match option {
        Some(t) => format!("{}", t),
        None => "".to_owned(),
    }
}
