use std::iter;

use im::{ordmap, OrdMap};

pub trait Key: Clone + Ord {}

impl<T> Key for T where T: Clone + Ord {}

pub trait Value<K>: Clone {
    fn intersects_with(&self, other: &Self) -> bool;

    fn union(&self, other: &Self) -> Self;

    fn key(&self) -> K;
}

// A data structure to maintain a minimal set of disjoint elements. It is implemented using a
// binary search tree.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct MergeSet<K, V>
where
    K: Key,
    V: Value<K>,
{
    tree: OrdMap<K, V>,
}

impl<K, V> MergeSet<K, V>
where
    K: Key,
    V: Value<K>,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            tree: OrdMap::new(),
        }
    }

    #[inline]
    pub fn new_with(initial_value: V) -> Self {
        let mut set = Self::new();
        set.insert(initial_value);
        set
    }
}

impl<K, V> Default for MergeSet<K, V>
where
    K: Key,
    V: Value<K>,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> From<Vec<V>> for MergeSet<K, V>
where
    K: Key,
    V: Value<K>,
{
    #[inline]
    fn from(vec: Vec<V>) -> Self {
        let mut set = MergeSet::new();
        set.extend(vec);
        set
    }
}

impl<K, V> MergeSet<K, V>
where
    K: Key,
    V: Value<K>,
{
    #[inline]
    pub fn insert(&mut self, mut item: V) {
        let mut priority = item.key();

        // Check for intersection with predecessor.
        let pred = self.tree.get_prev(&priority);
        if let Some((pred_pri, pred_v)) = pred {
            // If intersecting, merge and remove predecessor.
            // Set item's priority to that of predecessor.
            if item.intersects_with(pred_v) {
                item = item.union(pred_v);
                priority = pred_pri.clone();

                self.tree.remove(&priority);
            }
        }

        // Check for intersection with successor.
        let succ = self.tree.get_next(&priority);
        if let Some((succ_pri, succ_v)) = succ {
            // If intersecting, merge and remove successor.
            if item.intersects_with(succ_v) {
                item = item.union(succ_v);
                let del_pri = succ_pri.clone();
                self.tree.remove(&del_pri);
            }
        }

        self.tree.insert(priority, item);
    }

    #[inline]
    pub fn remove(&mut self, priority: K) -> Option<V> {
        self.tree.remove(&priority)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        self.tree.iter().into()
    }
}

impl<'a, K, V> IntoIterator for &'a MergeSet<K, V>
where
    K: Key,
    V: Value<K>,
{
    type Item = &'a V;
    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.tree.iter().into()
    }
}

impl<'a, K, V> IntoIterator for MergeSet<K, V>
where
    K: Key,
    V: Value<K>,
{
    type Item = V;
    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.tree.into_iter().into()
    }
}

impl<K, V> Extend<V> for MergeSet<K, V>
where
    K: Key,
    V: Value<K>,
{
    #[inline]
    fn extend<I: IntoIterator<Item = V>>(&mut self, iter: I) {
        for v in iter {
            self.insert(v);
        }
    }
}

impl<K, V> iter::FromIterator<V> for MergeSet<K, V>
where
    K: Key,
    V: Value<K>,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        let mut set = Self::new();
        set.extend(iter);
        set
    }
}

pub struct Iter<'a, K, V> {
    inner: ordmap::Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Key,
    V: Clone,
{
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, v)| v)
    }
}

impl<'a, K, V> From<ordmap::Iter<'a, K, V>> for Iter<'a, K, V>
where
    K: Key,
    V: Clone,
{
    #[inline]
    fn from(inner: ordmap::Iter<'a, K, V>) -> Self {
        Self { inner }
    }
}

pub struct IntoIter<K, V> {
    inner: ordmap::ConsumingIter<(K, V)>,
}

impl<K, V> Iterator for IntoIter<K, V>
where
    K: Key,
    V: Clone,
{
    type Item = V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, v)| v)
    }
}

impl<K, V> From<ordmap::ConsumingIter<(K, V)>> for IntoIter<K, V>
where
    K: Key,
    V: Clone,
{
    #[inline]
    fn from(inner: ordmap::ConsumingIter<(K, V)>) -> Self {
        Self { inner }
    }
}
