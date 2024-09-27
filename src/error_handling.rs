use core::str;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::Debug;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::{borrow::Cow, ops::Range};

use wutil::Span;

mod renderer;

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
    /// The ANSII escape sequence to use
    escape: &'static str,
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
        let mut renderer = renderer::DiagnosticRenderer::new(code, &self.hints, &self.msg);

        for hint in &self.hints {
            renderer.render_hint(hint);
        }

        renderer.finish()
    }

    pub fn prepend(&mut self, prefix: &str) {
        let mut msg = "".into();
        mem::swap(&mut msg, &mut self.msg);

        let mut msg = msg.into_owned();
        msg.insert_str(0, prefix);

        self.msg = msg.into();
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
            escape: "\x1b[31m", // Red
        }
    }
    pub fn new_warning<M>(msg: M, span: Span) -> Self
    where
        M: Into<Cow<'static, str>>,
    {
        Self {
            msg: msg.into(),
            span: span.into(),
            pointer_char: '~',
            escape: "\x1b[33m", // Yellow
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
            escape: "\x1b[36m", // Cyan
        }
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
