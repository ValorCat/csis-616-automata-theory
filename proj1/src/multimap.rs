use std::cmp::Eq;
use std::hash::Hash;
use std::collections::HashMap;
use std::collections::HashSet;

/// Represents a multimap, a 1 to many map
/// Duplicate key-value mappings are ignored
pub type MultiMap<K, V> = HashMap<K, HashSet<V>>;

/// This trait seems conceptually redundant to me, but it seems like Rust
/// needs this layer of indirection because MultiMap is generic
pub trait MultiMapMethods<K, V> {

    /// Get the set of values associated with a key
    fn get_multi(&self, key: K) -> HashSet<V>;

    /// Add a mapping from a key to a value
    fn add_multi(&mut self, key: K, value: V);

    /// Add a set of mappings from a key to a value
    fn add_all_multi(&mut self, key: K, values: &HashSet<V>);
}

impl<K, V> MultiMapMethods<K, V> for MultiMap<K, V> where
        K: Copy + Eq + Hash,
        V: Clone + Eq + Hash {

    fn get_multi(&self, key: K) -> HashSet<V> {
        match self.get(&key) {
            Some(set) => set.clone(),
            None => HashSet::new()
        }
    }

    fn add_multi(&mut self, key: K, value: V) {
        self.entry(key).or_default().insert(value);
    }

    fn add_all_multi(&mut self, key: K, values: &HashSet<V>) {
        let set = self.entry(key).or_default();
        values.iter().for_each(|v| { set.insert(v.clone()); });
    }
}

/// Compute the union of a list of multimaps
pub fn union_multi<K, V>(maps: &[&MultiMap<K, V>]) -> MultiMap<K, V> where
        K: Copy + Eq + Hash,
        V: Clone + Eq + Hash {
    let mut union: MultiMap<K, V> = HashMap::new();
    for &map in maps {
        for (key, values) in map {
            for value in values {
                union.add_multi(*key, value.clone());
            }
        }
    }
    union
}
