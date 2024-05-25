#![allow(dead_code)]

use std::{mem::MaybeUninit, ops::Range, ptr::addr_of, usize};

pub trait RangeExt {
    fn overlaps_with(&self, other: &Self) -> bool;
}

impl RangeExt for Range<usize> {
    fn overlaps_with(&self, other: &Self) -> bool {
        self.contains(&other.start)
            || self.contains(&(other.end - 1))
            || other.contains(&self.start)
            || other.contains(&(self.end - 1))
    }
}

pub trait IterExt {
    type Item;

    /// Gets N items from an iterator and returns them as an array. Otherwise returns `None`.
    fn collect_n<const N: usize>(&mut self) -> Option<[Self::Item; N]>;
}

impl<I: Iterator> IterExt for I {
    type Item = I::Item;

    /// Gets N items from an iterator and returns them as an array. Otherwise returns `None`.
    fn collect_n<const N: usize>(&mut self) -> Option<[Self::Item; N]> {
        let mut arr: [MaybeUninit<Self::Item>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        for i in 0..N {
            let Some(item) = self.next() else {
                for item in &mut arr[0..i] {
                    unsafe { item.assume_init_drop() };
                }
                return None;
            };

            arr[i].write(item);
        }

        Some(unsafe { addr_of!(arr).cast::<[Self::Item; N]>().read() })
    }
}

pub trait StrExt {
    /// Gets the position of a substring within a string.
    fn substr_pos(&self, substr: &Self) -> Option<Range<usize>>;
    /// Gets the length of the character at `byte_index`
    fn char_length(&self, byte_index: usize) -> Option<usize>;
    /// Gets the range of the character at `byte_index`
    fn char_range(&self, byte_index: usize) -> Option<Range<usize>>;
}

impl StrExt for str {
    fn substr_pos(&self, substr: &Self) -> Option<Range<usize>> {
        let self_start = self.as_ptr() as usize;
        let substr_start = substr.as_ptr() as usize;

        let pos_start = substr_start.wrapping_sub(self_start);

        if pos_start >= self.len() {
            return None;
        }

        Some(pos_start..pos_start + substr.len())
    }

    fn char_length(&self, byte_index: usize) -> Option<usize> {
        let subsl = self.get(byte_index..)?;
        let mut iter = subsl.char_indices();
        iter.next()?;

        Some(iter.next().map(|s| s.0).unwrap_or(subsl.len()))
    }
    fn char_range(&self, byte_index: usize) -> Option<Range<usize>> {
        Some(byte_index..byte_index + self.char_length(byte_index)?)
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
