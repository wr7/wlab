use std::{mem::MaybeUninit, ops::Deref, slice};

/// A stack-allocated resizable array with a max size of `N`.
pub(crate) struct MaxVec<V, const N: usize> {
    pub(crate) storage: [MaybeUninit<V>; N],
    pub(crate) len: usize,
}

impl<V, const N: usize> MaxVec<V, N> {
    pub fn new() -> Self {
        Self {
            storage: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }

    /// Pushes to the end. If the `MaxVec` is full, an element is popped from the front and returned.
    pub fn cycle(&mut self, elem: V) -> Option<V> {
        if N == 0 {
            panic!()
        }

        let ret_val = if self.len >= N {
            unsafe {
                let val = Some(self.storage[0].assume_init_read());
                self.len = N - 1;

                core::ptr::copy(
                    self.storage.as_ptr().cast::<V>().offset(1),
                    self.storage.as_mut_ptr().cast::<V>(),
                    self.len,
                );

                val
            }
        } else {
            None
        };

        self.storage[self.len] = MaybeUninit::new(elem);
        self.len += 1;

        ret_val
    }
}

impl<V, const N: usize> AsRef<[V]> for MaxVec<V, N> {
    fn as_ref(&self) -> &[V] {
        let ptr = self.storage.as_ptr().cast::<V>();
        unsafe { slice::from_raw_parts(ptr, self.len) }
    }
}

impl<V, const N: usize> Deref for MaxVec<V, N> {
    type Target = [V];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
