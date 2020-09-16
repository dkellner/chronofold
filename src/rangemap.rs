use std::borrow::Borrow;
use std::collections::BTreeMap;

/// A map containing values for ranges of keys (i.e. `key..`).
#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct RangeFromMap<K: Ord, V> {
    #[cfg_attr(feature = "serde", serde(flatten))]
    map: BTreeMap<K, V>,
}

impl<K: Ord, V> RangeFromMap<K, V> {
    pub(crate) fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub(crate) fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.map.range(..=key).map(|(_, v)| v).next_back()
    }
}

impl<K: Ord, V: Eq> RangeFromMap<K, V> {
    /// Sets a key-value pair.
    ///
    /// This does not perform any compaction. This means that `set(20, 1)` and
    /// later `set(10, 1)` will lead to two entries in the inner map, while
    /// `set(10, 1)` and later `set(20, 1)` results in just one entry.
    ///
    /// However, in this crate we only set keys that are greater than all
    /// existing keys. This keeps the internal representation of the range map
    /// minimal.
    pub(crate) fn set(&mut self, key: K, value: V) {
        if self.get(&key) != Some(&value) {
            self.map.insert(key, value);
        }
    }
}

impl<K: Ord, V> Default for RangeFromMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Map = RangeFromMap<usize, &'static str>;

    #[test]
    fn get_empty() {
        let map = Map::new();
        assert_eq!(None, map.get(&0));
    }

    #[test]
    fn set_and_get() {
        let mut map = Map::new();
        map.set(10, "alice");
        assert_eq!(None, map.get(&5));
        assert_eq!(Some(&"alice"), map.get(&10));
        assert_eq!(Some(&"alice"), map.get(&15));
    }

    #[test]
    fn test_missing_compaction() {
        let mut m1 = RangeFromMap::<usize, usize>::new();
        let mut m2 = RangeFromMap::<usize, usize>::new();
        m1.set(20, 2);
        m2.set(20, 2);
        assert_eq!(m1, m2);

        m1.set(10, 1);
        m1.set(15, 1);
        m2.set(15, 1);
        m2.set(10, 1);
        assert_ne!(m1, m2);
    }
}
