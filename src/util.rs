#![allow(dead_code)]

use std::ops::Range;

pub trait StrExt {
    /// Gets the index of a substring in a string
    fn substr_range(&self, item: &str) -> Option<Range<usize>>;
}

pub trait SliceExt {
    type Item;
    fn subslice_range(&self, item: &[Self::Item]) -> Option<Range<usize>>;
    fn elem_offset(&self, item: &Self::Item) -> Option<usize>;
}

impl StrExt for str {
    fn substr_range(&self, substr: &str) -> Option<Range<usize>> {
        self.as_bytes().subslice_range(substr.as_bytes())
    }
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

    return line_no;
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

    return col_no;
}
