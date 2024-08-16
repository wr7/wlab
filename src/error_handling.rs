use core::str;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::{borrow::Cow, ops::Range};

use wutil::Span;

use crate::util;

/// Includes information about where something appears in a source file
#[derive(Debug, Clone, Copy)]
pub struct Spanned<T>(pub T, pub Span);

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Spanned<T> {
    pub fn as_sref(&self) -> Spanned<&T> {
        Spanned(&self.0, self.1)
    }
}

/// Gets the span of a slice of objects in the source file.
///
/// Returns `None` if the slice is empty.
#[must_use]
pub fn span_of<T>(slice: &[Spanned<T>]) -> Option<Span> {
    let (first, last) = slice.first().zip(slice.last())?;

    Some(first.1.span_at().with_end(last.1.end))
}

impl<T: Eq> Eq for Spanned<T> {}

#[derive(Clone)]
pub struct Hint {
    msg: Cow<'static, str>,
    span: Range<usize>,
    pointer_char: char,
}

#[derive(Clone)]
pub struct Diagnostic {
    pub msg: Cow<'static, str>,
    pub hints: Vec<Hint>,
}

/// Creates a diagnostic with a message and set of hints
#[macro_export]
macro_rules! diagnostic {
    ($msg:expr, [$($hint:expr),* $(,)?] $(,)?) => {
        $crate::error_handling::Diagnostic {msg: $msg.into(), hints: ::std::vec![$($hint),*]}
    };
}

impl Diagnostic {
    pub fn render(&self, code: &str) -> String {
        let mut ret_val = "\n ".to_owned();

        ret_val += &self.msg;
        ret_val.push('\n');

        for _ in 0..ret_val.len() {
            ret_val.push('-');
        }

        ret_val.push('\n');

        for (i, hint) in self.hints.iter().enumerate() {
            ret_val += &hint.render_snippet(code);

            if i != self.hints.len() - 1 {
                ret_val += "\n ...\n";
            }
        }

        ret_val.push('\n');

        ret_val
    }
}

impl Hint {
    pub fn new_error<M>(msg: M, span: Span) -> Self
    where
        M: Into<Cow<'static, str>>,
    {
        Self {
            msg: msg.into(),
            span: span.into(),
            pointer_char: '^',
        }
    }
    pub fn new_info<M>(msg: M, span: Span) -> Self
    where
        M: Into<Cow<'static, str>>,
    {
        Self {
            msg: msg.into(),
            span: span.into(),
            pointer_char: '-',
        }
    }
    fn render_snippet(&self, code: &str) -> String {
        let (line_st, col_st) = util::line_and_col(code, self.span.start);
        let (line_end, col_end) =
            util::line_and_col(code, self.span.end.saturating_sub(1).max(self.span.start));

        let mut ret_val = String::new();

        for (i, line) in code
            .lines()
            .enumerate()
            .map(|(i, l)| (i + 1, l))
            .take(line_end)
            .skip(line_st.saturating_sub(3))
        {
            // Print line number, line, and gutter //
            let padding = line_end.ilog10() - i.ilog10();

            for _ in 0..padding {
                ret_val += " ";
            }

            ret_val += &i.to_string();
            ret_val += " | ";
            ret_val += line;
            ret_val += "\n";

            if !(line_st..=line_end).contains(&i) || line.is_empty() {
                continue;
            }

            // Print pointer gutter //

            for _ in 0..=line_end.ilog10() {
                ret_val += " ";
            }

            ret_val += " | ";

            // Print pointer

            let arrow_st = if i == line_st { col_st } else { 1 };
            let arrow_end = if i == line_end {
                col_end
            } else {
                line.chars().count()
            };

            for _ in 0..arrow_st - 1 {
                ret_val += " ";
            }

            for _ in arrow_st..=arrow_end.max(arrow_st) {
                ret_val.push(self.pointer_char);
            }

            ret_val += "\n";
        }

        if !self.msg.is_empty() {
            // Print message //
            for _ in 0..=line_end.ilog10() {
                ret_val += " ";
            }
            ret_val += " | ";
            ret_val += &self.msg;
        }

        ret_val
    }
}

impl<T> AsRef<T> for Spanned<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for Spanned<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> Borrow<T> for Spanned<T> {
    fn borrow(&self) -> &T {
        self.as_ref()
    }
}

impl<T> BorrowMut<T> for Spanned<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}
