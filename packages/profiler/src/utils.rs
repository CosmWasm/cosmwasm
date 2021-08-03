use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

/// For KV stores that keep vectors/vecdeques as their values, provides a way
/// to quickly push to a vector under a given key. If the vector doesn't exist
/// yet, it's created.
pub trait InsertPush<K: Eq + Hash, I> {
    fn insert_push(&mut self, k: K, item: I);
}

impl<K: Eq + Hash + Clone, I> InsertPush<K, I> for HashMap<K, Vec<I>> {
    fn insert_push(&mut self, k: K, item: I) {
        if let None = self.get(&k) {
            self.insert(k.clone(), Vec::new());
        }

        let q = self.get_mut(&k).unwrap();
        q.push(item);
    }
}

impl<K: Eq + Hash + Clone, I> InsertPush<K, I> for HashMap<K, VecDeque<I>> {
    fn insert_push(&mut self, k: K, item: I) {
        if let None = self.get(&k) {
            self.insert(k.clone(), VecDeque::new());
        }

        let q = self.get_mut(&k).unwrap();
        q.push_back(item);
    }
}
