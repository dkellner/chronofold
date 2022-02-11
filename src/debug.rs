use std::fmt;

use crate::{Author, Chronofold, LogIndex};

impl<A: Author, T: fmt::Debug> Chronofold<A, T> {
    pub fn formatted_log(&self) -> String {
        let mut result = format!(
            "{:<4} | {:<4} | {:<4} | {:<4} | change\n",
            "idx", "ref", "next", "del"
        );
        for (idx, change) in self.log.iter().enumerate() {
            let idx = LogIndex(idx);
            let ref_ = format_option(self.references.get(&idx));
            let next = format_option(self.next_indices.get(&idx));
            let del = format_option(change.1);
            let change = &change.0;
            result += &format!("{idx:<4} | {ref_:<4} | {next:<4} | {del:<4} | {change:?}\n");
        }
        result
    }
}

fn format_option<T: fmt::Display>(option: Option<T>) -> String {
    match option {
        Some(t) => format!("{t}"),
        None => "".to_owned(),
    }
}
