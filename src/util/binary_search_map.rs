use std::borrow::Borrow;

pub(crate) struct BinarySearchMap<K, V> {
    pub(crate) map: Vec<(K, V)>,
}

impl<K, V> Default for BinarySearchMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl<K, V> BinarySearchMap<K, V> {
    pub const fn new() -> Self {
        Self { map: Vec::new() }
    }

    pub fn get<'a, Q>(&'a self, key: &Q) -> Option<&'a V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.get_pair(key).map(|(_, v)| v)
    }

    /// Inserts a given key and value.
    ///
    /// Upon success, the index of the value is returned.
    /// If that key already exists, the index of it is returned.
    pub fn insert(&mut self, key: K, val: V) -> Result<usize, usize>
    where
        K: Ord,
    {
        match self.index_of(&key) {
            Ok(idx) => Err(idx),
            Err(idx) => {
                self.insert_at(idx, key, val);
                Ok(idx)
            }
        }
    }

    pub fn insert_at(&mut self, idx: usize, key: K, val: V) {
        self.map.insert(idx, (key, val));
    }

    pub fn get_or_insert_with<'a, F, Q>(&'a mut self, key: &Q, func: F) -> &'a V
    where
        K: Borrow<Q>,
        Q: Ord + ToOwned<Owned = K> + ?Sized,
        F: FnOnce() -> V,
    {
        let idx;
        match self.index_of(key) {
            Ok(i) => idx = i,
            Err(i) => {
                idx = i;
                self.map.insert(i, (key.to_owned(), (func)()));
            }
        }

        &self.map[idx].1
    }

    pub fn index_of<Q>(&self, key: &Q) -> Result<usize, usize>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.map.binary_search_by(|(k, _)| k.borrow().cmp(key))
    }

    pub fn pair_at(&self, idx: usize) -> (&K, &V) {
        let (k, v) = &self.map[idx];
        (k, v)
    }

    pub fn val_at(&self, idx: usize) -> &V {
        self.pair_at(idx).1
    }

    pub fn get_pair<'a, Q>(&'a self, key: &Q) -> Option<(&'a K, &'a V)>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.index_of(key)
            .ok()
            .and_then(|idx| self.map.get(idx))
            .map(|(k, v)| (k, v))
    }
}
