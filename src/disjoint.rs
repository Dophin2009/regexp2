use std::iter;
use tree::map;
use tree::Map;

pub trait Intersect {
    fn intersect(&self, other: &Self) -> bool;

    fn union(&self, other: &Self) -> Self;
}

pub trait Priority<K: Ord> {
    fn priority(&self) -> K;
}

// A data structure to maintain a minimal set of disjoint elements. It is implemented using a
// binary search tree.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DisjointSet<K, V>
where
    K: Clone + Ord,
    V: Intersect + Priority<K>,
{
    tree: Map<K, V>,
}

impl<K, V> DisjointSet<K, V>
where
    K: Clone + Ord,
    V: Intersect + Priority<K>,
{
    pub fn new() -> Self {
        Self { tree: Map::new() }
    }

    pub fn new_with(initial_value: V) -> Self {
        let mut set = Self::new();
        set.insert(initial_value);
        set
    }
}

impl<K, V> From<Vec<V>> for DisjointSet<K, V>
where
    K: Clone + Ord,
    V: Intersect + Priority<K>,
{
    fn from(vec: Vec<V>) -> Self {
        let mut set = DisjointSet::new();
        set.extend(vec);
        set
    }
}

impl<K, V> DisjointSet<K, V>
where
    K: Clone + Ord,
    V: Intersect + Priority<K>,
{
    pub fn insert(&mut self, mut item: V) {
        let mut priority = item.priority();

        // Check for intersection with predecessor.
        let pred = self.tree.pred(&priority, true);
        if let Some((pred_pri, pred_v)) = pred {
            // If intersecting, merge and remove predecessor.
            // Set item's priority to that of predecessor.
            if item.intersect(pred_v) {
                item = item.union(pred_v);
                priority = pred_pri.clone();

                self.tree.remove(&priority);
            }
        }

        // Check for intersection with successor.
        let succ = self.tree.succ(&priority, true);
        if let Some((succ_pri, succ_v)) = succ {
            // If intersecting, merge and remove successor.
            if item.intersect(succ_v) {
                item = item.union(succ_v);
                let del_pri = succ_pri.clone();
                self.tree.remove(&del_pri);
            }
        }

        self.tree.insert(priority, item);
    }

    pub fn remove(&mut self, priority: K) -> Option<V> {
        self.tree.remove(&priority).and_then(|(_, v)| Some(v))
    }

    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }
}

impl<K, V> DisjointSet<K, V>
where
    K: Clone + Ord,
    V: Intersect + Priority<K>,
{
    pub fn iter(&self) -> Iter<K, V> {
        self.tree.iter().into()
    }

    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        self.tree.iter_mut().into()
    }
}

impl<'a, K, V> IntoIterator for &'a DisjointSet<K, V>
where
    K: Clone + Ord,
    V: Intersect + Priority<K>,
{
    type Item = &'a V;
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.tree.iter().into()
    }
}

impl<'a, K, V> IntoIterator for &'a mut DisjointSet<K, V>
where
    K: Clone + Ord,
    V: Intersect + Priority<K>,
{
    type Item = &'a mut V;
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.tree.iter_mut().into()
    }
}

impl<'a, K, V> IntoIterator for DisjointSet<K, V>
where
    K: Clone + Ord,
    V: Intersect + Priority<K>,
{
    type Item = V;
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.tree.into_iter().into()
    }
}

impl<K, V> Extend<V> for DisjointSet<K, V>
where
    K: Clone + Ord,
    V: Intersect + Priority<K>,
{
    fn extend<I: IntoIterator<Item = V>>(&mut self, iter: I) {
        for v in iter {
            self.insert(v);
        }
    }
}

impl<K, V> iter::FromIterator<V> for DisjointSet<K, V>
where
    K: Clone + Ord,
    V: Intersect + Priority<K>,
{
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        let mut set = Self::new();
        set.extend(iter);
        set
    }
}

pub struct Iter<'a, K, V> {
    map_iter: map::Iter<'a, K, V>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.map_iter.next().and_then(|(_, v)| Some(v))
    }
}

impl<'a, K, V> From<map::Iter<'a, K, V>> for Iter<'a, K, V> {
    fn from(map_iter: map::Iter<'a, K, V>) -> Self {
        Self { map_iter }
    }
}

pub struct IterMut<'a, K, V> {
    map_iter: map::IterMut<'a, K, V>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = &'a mut V;

    fn next(&mut self) -> Option<Self::Item> {
        self.map_iter.next().and_then(|(_, v)| Some(v))
    }
}

impl<'a, K, V> From<map::IterMut<'a, K, V>> for IterMut<'a, K, V> {
    fn from(map_iter: map::IterMut<'a, K, V>) -> Self {
        Self { map_iter }
    }
}

pub struct IntoIter<K, V> {
    map_iter: map::IntoIter<K, V>,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.map_iter.next().and_then(|(_, v)| Some(v))
    }
}

impl<'a, K, V> From<map::IntoIter<K, V>> for IntoIter<K, V> {
    fn from(map_iter: map::IntoIter<K, V>) -> Self {
        Self { map_iter }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Intersect for bool {
        fn intersect(&self, _: &Self) -> bool {
            false
        }

        fn union(&self, _: &Self) -> Self {
            true
        }
    }

    impl Priority<u32> for bool {
        fn priority(&self) -> u32 {
            0
        }
    }

    #[test]
    fn test_works() {
        let mut set = DisjointSet::new();
        set.insert(true);
    }
}
