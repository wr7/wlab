use std::{
    fmt::Debug,
    ops::{Index, IndexMut, Range},
};

/// Represents a range of text. This is equivalent to rust's Range<usize> but has better properties.
#[derive(Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn span_after(self) -> Self {
        (self.end..self.end).into()
    }

    pub fn span_at(self) -> Self {
        (self.start..self.start).into()
    }

    pub fn at(pos: usize) -> Self {
        (pos..pos).into()
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Range<usize> as Debug>::fmt(&(*self).into(), f)
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Self {
        span.start..span.end
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl Index<Span> for str {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.start..index.end]
    }
}

impl IndexMut<Span> for str {
    fn index_mut(&mut self, index: Span) -> &mut Self::Output {
        &mut self[index.start..index.end]
    }
}

impl<T> Index<Span> for [T] {
    type Output = [T];

    fn index(&self, index: Span) -> &Self::Output {
        &self[index.start..index.end]
    }
}

impl<T> IndexMut<Span> for [T] {
    fn index_mut(&mut self, index: Span) -> &mut Self::Output {
        &mut self[index.start..index.end]
    }
}
