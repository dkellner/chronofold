use std::collections::BTreeMap;

/// A map from `K` to `K` with a default value of `O::default().add(key)`.
///
/// For a chronofold, there are two cases where a structure like this makes
/// sense: storing the next index in the weave (current index + 1 for
/// consecutive inserts) and the reference (current index - 1 for the same
/// case).
///
/// This implementation ensures that a value of `K + O::default()` is *never*
/// stored. That's the reason you will not get a mutable borrow to any map's
/// value, but have to use `set()` to insert/manipulate values.
#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct OffsetMap<K: Ord, O> {
    map: BTreeMap<K, Option<O>>,
}

pub(crate) trait Offset<K>: Default {
    fn add(&self, value: &K) -> K;
    fn sub(a: &K, b: &K) -> Self;
}

impl<K: Ord, O> OffsetMap<K, O> {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
}

impl<K: Ord, O: Offset<K>> OffsetMap<K, O> {
    pub fn get(&self, key: &K) -> Option<K> {
        match self.map.get(key) {
            Some(None) => None,
            Some(Some(offset)) => Some(offset.add(key)),
            None => Some(O::default().add(key)),
        }
    }

    pub fn set(&mut self, key: K, value: Option<K>) {
        if let Some(value) = value {
            if O::default().add(&key) == value {
                self.map.remove(&key);
            } else {
                let offset = O::sub(&value, &key);
                self.map.insert(key, Some(offset));
            }
        } else {
            self.map.insert(key, None);
        }
    }
}

impl<K: Ord, O> Default for OffsetMap<K, O> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{index::RelativeNextIndex, LogIndex};

    type IdxMap = OffsetMap<LogIndex, RelativeNextIndex>;

    #[test]
    fn get_default() {
        let map = IdxMap::new();
        assert_eq!(Some(LogIndex(1)), map.get(&LogIndex(0)));
    }

    #[test]
    fn set_default() {
        let mut map = IdxMap::new();
        map.set(LogIndex(1), Some(LogIndex(2)));
        assert_eq!(Some(LogIndex(2)), map.get(&LogIndex(1)));
        assert_eq!(IdxMap::new(), map); // the default is not stored
    }

    #[test]
    fn set_and_get_none() {
        let mut map = IdxMap::new();
        map.set(LogIndex(42), None);
        assert_eq!(None, map.get(&LogIndex(42)));
    }

    #[test]
    fn set_and_get_value() {
        let mut map = IdxMap::new();
        map.set(LogIndex(42), Some(LogIndex(50)));
        map.set(LogIndex(50), Some(LogIndex(1)));
        assert_eq!(Some(LogIndex(50)), map.get(&LogIndex(42)));
        assert_eq!(Some(LogIndex(1)), map.get(&LogIndex(50)));
    }
}
