use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub struct Table<T, U, V>
where
    T: Eq + Hash,
    U: Eq + Hash,
{
    map: HashMap<T, HashMap<U, V>>,
}

impl<T, U, V> Table<T, U, V>
where
    T: Eq + Hash,
    U: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, row: T, col: U, val: V) -> Option<V> {
        match self.map.get_mut(&row) {
            Some(c) => c.insert(col, val),
            None => {
                let mut map = HashMap::new();
                map.insert(col, val);
                self.map.insert(row, map);
                None
            }
        }
    }

    pub fn set_or<F>(&mut self, row: T, col: U, val: V, or: F)
    where
        F: FnOnce(&mut V),
    {
        match self.get_mut(&row, &col) {
            Some(v) => or(v),
            None => {
                self.set(row, col, val);
            }
        };
    }

    pub fn get_mut(&mut self, row: &T, col: &U) -> Option<&mut V> {
        match self.map.get_mut(row) {
            Some(c) => c.get_mut(col),
            None => None,
        }
    }

    pub fn get(&self, row: &T, col: &U) -> Option<&V> {
        match self.map.get(row) {
            Some(c) => c.get(col),
            None => None,
        }
    }
}

impl<'a, T, U, V> IntoIterator for &'a Table<T, U, V>
where
    T: Copy + Eq + Hash,
    U: Eq + Hash,
{
    type Item = (&'a T, &'a U, &'a V);
    type IntoIter = TableIterator<&'a T, &'a U, &'a V>;

    fn into_iter(self) -> Self::IntoIter {
        let vec: Vec<(&'a T, &'a U, &'a V)> = self
            .map
            .iter()
            .flat_map(|(row, c)| c.iter().map(move |(col, val)| (row, col, val)))
            .collect();
        TableIterator(vec.into_iter())
    }
}

pub struct TableIterator<T, U, V>(std::vec::IntoIter<(T, U, V)>)
where
    T: Eq + Hash,
    U: Eq + Hash;

impl<T, U, V> Iterator for TableIterator<T, U, V>
where
    T: Eq + Hash,
    U: Eq + Hash,
{
    type Item = (T, U, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
