use std::{borrow::Borrow, collections::HashMap, hash::Hash};

pub trait HashMapExt {
    type Key;
    type Val;

    fn get_or_insert_with_mut<'a, Q, F>(&'a mut self, k: &Q, f: F) -> &'a mut Self::Val
    where
        Self::Key: Borrow<Q>,
        Q: ToOwned<Owned = Self::Key> + Hash + Eq + ?Sized,
        F: FnOnce() -> Self::Val;
}

impl<K, V> HashMapExt for HashMap<K, V>
where
    K: Hash + Eq,
{
    type Key = K;
    type Val = V;

    fn get_or_insert_with_mut<'a, Q, F>(&'a mut self, k: &Q, f: F) -> &'a mut V
    where
        K: Borrow<Q>,
        Q: ToOwned<Owned = K> + Hash + Eq + ?Sized,
        F: FnOnce() -> V,
    {
        if self.get(k).is_none() {
            self.insert(k.to_owned(), f());
        }

        let Some(val) = self.get_mut(k) else {
            unreachable!()
        };

        val
    }
}
