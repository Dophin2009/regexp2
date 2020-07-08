use std::collections::HashMap;
use std::hash::Hash;

/// A two-way lookup table.
#[derive(Debug)]
pub struct Table<T, U, V>
where
    T: Eq + Hash,
    U: Eq + Hash,
{
    /// Implemented using nested hashmaps; kinda ugly. Could probably be improved.
    map: HashMap<T, HashMap<U, V>>,
}

impl<T, U, V> Table<T, U, V>
where
    T: Eq + Hash,
    U: Eq + Hash,
{
    /// Create an empty table.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Set the value in the table with the given keys.
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

    /// Set the value in the table with the given keys, or if some value already exists for those
    /// keys, execute the given callback.
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

    /// Retrieve a mutable reference to the value in the table with the given keys.
    pub fn get_mut(&mut self, row: &T, col: &U) -> Option<&mut V> {
        match self.map.get_mut(row) {
            Some(c) => c.get_mut(col),
            None => None,
        }
    }

    /// Retrieve an immutable reference to the value in the table with the given keys.
    pub fn get(&self, row: &T, col: &U) -> Option<&V> {
        match self.map.get(row) {
            Some(c) => c.get(col),
            None => None,
        }
    }

    /// Retrieve an immutable reference to a row of values.
    pub fn get_row(&self, row: &T) -> HashMap<&U, &V> {
        let row_map = match self.map.get(row) {
            Some(m) => m,
            None => return HashMap::new(),
        };

        row_map.iter().collect()
    }

    /// Retrieve an immutable reference to a column of values.
    pub fn get_col(&self, col: &U) -> HashMap<&T, &V> {
        let mut result = HashMap::new();
        for (row, column_map) in self.map.iter() {
            for (column_key, val) in column_map.iter() {
                if *column_key == *col {
                    result.insert(row, val);
                }
            }
        }
        result
    }
}

impl<T, U, V> Clone for Table<T, U, V>
where
    T: Clone + Eq + Hash,
    U: Clone + Eq + Hash,
    V: Clone,
{
    /// Clone the table.
    fn clone(&self) -> Self {
        Table {
            map: self.map.clone(),
        }
    }
}

impl<'a, T, U, V> IntoIterator for &'a Table<T, U, V>
where
    T: Clone + Eq + Hash,
    U: Eq + Hash,
{
    type Item = (&'a T, &'a U, &'a V);
    type IntoIter = TableIterator<&'a T, &'a U, &'a V>;

    /// Produce an iterator on all the values in the table. See [TableIterator].
    fn into_iter(self) -> Self::IntoIter {
        let vec: Vec<(&'a T, &'a U, &'a V)> = self
            .map
            .iter()
            .flat_map(|(row, c)| c.iter().map(move |(col, val)| (row, col, val)))
            .collect();
        TableIterator(vec.into_iter())
    }
}

/// An iterator on the the values stored in the table. Each item is a tuple consisting of each
/// set of keys and value.
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
