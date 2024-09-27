use std::ops::Range;

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
