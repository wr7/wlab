use std::cell::UnsafeCell;

/// A restricted `std::vec::Vec` that can be pushed to with an immutable reference
#[repr(transparent)]
pub struct PushVec<T> {
    inner: UnsafeCell<Vec<T>>,
}

impl<T> PushVec<T> {
    pub fn new() -> Self {
        Self {
            inner: UnsafeCell::new(Vec::new()),
        }
    }

    #[allow(unused)]
    pub fn from_mut(vec: &mut Vec<T>) -> &mut PushVec<T> {
        unsafe { wutil::transmute_mut::<Vec<T>, PushVec<T>>(vec) }
    }

    pub fn read(&mut self) -> &mut Vec<T> {
        self.inner.get_mut()
    }

    pub fn push(&self, val: T) {
        unsafe { (&mut *self.inner.get()).push(val) }
    }

    #[allow(unused)]
    pub fn into_inner(self) -> Vec<T> {
        self.inner.into_inner()
    }
}

impl<T> From<Vec<T>> for PushVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self {
            inner: UnsafeCell::new(value),
        }
    }
}

impl<T> From<PushVec<T>> for Vec<T> {
    fn from(value: PushVec<T>) -> Self {
        value.into_inner()
    }
}
