use std::{
    borrow::Borrow,
    cell::UnsafeCell,
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    mem,
    ops::{Deref, Range},
};

mod intersperse;
pub use intersperse::Intersperse;

/// A dynamically-sized array. If only one element is present, it is stored directly, and there is no indirection
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct MaybeVec<V> {
    inner: MaybeVecInner<V>,
}

impl<V> Debug for MaybeVec<V>
where
    V: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<V> MaybeVec<V> {
    pub const fn new() -> Self {
        Self {
            inner: MaybeVecInner::Vec(Vec::new()),
        }
    }

    /// Creates a `MaybeVec` with only a single element. This does not create an allocation
    pub const fn of(val: V) -> Self {
        Self {
            inner: MaybeVecInner::Val(val),
        }
    }

    pub fn push(&mut self, val: V) {
        let mut inner = MaybeVecInner::Vec(Vec::new());
        mem::swap(&mut inner, &mut self.inner);

        match inner {
            MaybeVecInner::Vec(ref mut v) => {
                if (**v).is_empty() {
                    self.inner = MaybeVecInner::Val(val);
                } else {
                    v.push(val);
                    mem::swap(&mut self.inner, &mut inner);
                }
            }
            MaybeVecInner::Val(v) => {
                self.inner = MaybeVecInner::Vec(vec![v, val]);
            }
        }
    }
}

impl<V> AsRef<[V]> for MaybeVec<V> {
    fn as_ref(&self) -> &[V] {
        match &self.inner {
            MaybeVecInner::Vec(v) => v,
            MaybeVecInner::Val(v) => core::slice::from_ref(v),
        }
    }
}

impl<V> Deref for MaybeVec<V> {
    type Target = [V];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[derive(Clone, PartialEq, Eq)]
enum MaybeVecInner<V> {
    Vec(Vec<V>),
    Val(V),
}

pub(crate) struct BinarySearchMap<K, V> {
    map: Vec<(K, V)>,
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

pub trait SliceExt {
    type Item;
    fn subslice_range(&self, item: &[Self::Item]) -> Option<Range<usize>>;
    fn elem_offset(&self, item: &Self::Item) -> Option<usize>;
}

impl<T> SliceExt for [T] {
    type Item = T;

    fn subslice_range(&self, subslice: &[T]) -> Option<Range<usize>> {
        if core::mem::size_of::<T>() == 0 {
            panic!()
        }

        let self_start = self.as_ptr() as usize;
        let subslice_start = subslice.as_ptr() as usize;

        let byte_start = subslice_start.wrapping_sub(self_start);
        let start = byte_start / core::mem::size_of::<T>();
        let end = start + subslice.len();

        if byte_start % core::mem::size_of::<T>() != 0 {
            return None;
        }

        if start <= self.len() && end <= self.len() {
            Some(start..end)
        } else {
            None
        }
    }

    fn elem_offset(&self, element: &T) -> Option<usize> {
        let self_start = self.as_ptr() as usize;
        let elem_start = element as *const T as usize;

        if core::mem::size_of::<T>() == 0 {
            return (self_start == elem_start).then_some(0);
        }

        let byte_offset = elem_start.wrapping_sub(self_start);
        let offset = byte_offset / core::mem::size_of::<T>();

        if byte_offset % core::mem::size_of::<T>() != 0 {
            return None;
        }

        if offset < self.len() {
            Some(offset)
        } else {
            None
        }
    }
}

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

/// Gets the line and column number of a byte in some text
pub fn line_and_col(src: &str, byte_position: usize) -> (usize, usize) {
    (
        line_number(src, byte_position),
        column_number(src, byte_position),
    )
}

/// Gets the line number of a byte in some text
fn line_number(src: &str, byte_position: usize) -> usize {
    let mut line_no = 1;

    for (i, c) in src.char_indices() {
        if i >= byte_position {
            break;
        } else if c == '\n' {
            line_no += 1;
        }
    }

    line_no
}

/// Gets the column number of a byte in some text
fn column_number(src: &str, byte_position: usize) -> usize {
    let mut col_no = 1;

    for (i, c) in src.char_indices() {
        if i >= byte_position {
            break;
        }

        if c == '\n' {
            col_no = 1;
        } else {
            col_no += 1;
        }
    }

    col_no
}

pub struct MemoryStore<T> {
    store: UnsafeCell<Vec<*mut T>>,
}

impl<T> MemoryStore<T> {
    pub fn new() -> Self {
        Self {
            store: UnsafeCell::new(Vec::new()),
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn add(&self, item: T) -> &mut T {
        let ptr = Box::into_raw(Box::new(item));
        let store = unsafe { &mut *self.store.get() };

        store.push(ptr);

        unsafe { &mut *ptr }
    }
}

impl<T> Drop for MemoryStore<T> {
    fn drop(&mut self) {
        let mut store = Vec::new();
        mem::swap(&mut store, self.store.get_mut());

        for obj in store {
            unsafe { mem::drop(Box::from_raw(obj)) };
        }
    }
}
