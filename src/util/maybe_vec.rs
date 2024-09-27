use std::{fmt::Debug, mem, ops::Deref};

/// A dynamically-sized array. If only one element is present, it is stored directly, and there is no indirection
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct MaybeVec<V> {
    pub(crate) inner: MaybeVecInner<V>,
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
pub(crate) enum MaybeVecInner<V> {
    Vec(Vec<V>),
    Val(V),
}
