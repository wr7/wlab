use std::{cell::UnsafeCell, mem};

pub struct MemoryStore<T> {
    pub(crate) store: UnsafeCell<Vec<*mut T>>,
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
