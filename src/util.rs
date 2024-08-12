use std::{borrow::Borrow, cell::UnsafeCell, collections::HashMap, hash::Hash, mem, ops::Range};

mod intersperse;
pub use intersperse::Intersperse;

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
