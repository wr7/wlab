use core::str;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::Debug;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::{borrow::Cow, ops::Range};

use wutil::Span;

use crate::util::{self, MaxVec};

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
        let mut renderer = DiagnosticRenderer::new(code, &self.hints, &self.msg);

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

#[derive(Clone, Copy)]
struct ResolvedHintRef<'a> {
    hint: &'a Hint,
    lines: Span,
    columns: Span,
}

impl<'a> Deref for ResolvedHintRef<'a> {
    type Target = &'a Hint;

    fn deref(&self) -> &Self::Target {
        &self.hint
    }
}

struct DiagnosticRenderer<'a> {
    code: &'a str,
    output: String,
    padding: u32,
    backlog: MaxVec<ResolvedHintRef<'a>, 2>,
}

impl<'a> DiagnosticRenderer<'a> {
    pub fn new(code: &'a str, hints: &[Hint], diagnostic_msg: &str) -> Self {
        fn get_padding(code: &str, hints: &[Hint]) -> u32 {
            let mut max_byte_index = None;
            let mut min_byte_index = None;

            for hint in hints {
                max_byte_index = Some(max_byte_index.unwrap_or(hint.span.end).max(hint.span.end));
                min_byte_index = Some(min_byte_index.unwrap_or(hint.span.end).min(hint.span.end));
            }

            1 + if let Some((max_byte_index, min_byte_index)) = max_byte_index.zip(min_byte_index) {
                let (line_end, _) =
                    util::line_and_col(code, max_byte_index.saturating_sub(1).max(min_byte_index));
                line_end.ilog10()
            } else {
                0
            }
        }

        let padding = get_padding(code, hints);

        let mut output = "\n\x1b[m ".to_owned();

        output += diagnostic_msg;

        output += "\n\n";

        if let Some(hint) = hints.first() {
            let (line_start, _) = util::line_and_col(code, hint.span.start);

            if line_start > 3 {
                for _ in 0..padding {
                    output.push(' ');
                }
                output += " ...\n";
            }
        }

        Self {
            code,
            padding,
            output,
            backlog: MaxVec::new(),
        }
    }

    fn render_hint(&mut self, hint: &'a Hint) {
        let next_hint = hint.resolve(self.code);
        let prev_hint = self.backlog.cycle(next_hint);
        let hint = self.backlog[0];

        if self.backlog.len() <= 1 {
            return;
        }

        self.direct_render_hint(prev_hint, hint, Some(next_hint));
    }

    fn direct_render_hint(
        &mut self,
        prev_hint: Option<ResolvedHintRef>,
        hint: ResolvedHintRef,
        _next_hint: Option<ResolvedHintRef>,
    ) {
        let mut leading_lines_to_render = 2;

        if let Some(prev_hint) = prev_hint {
            for _ in 0..=self.padding {
                self.output.push(' ');
            }

            if hint.lines.start.wrapping_sub(prev_hint.lines.end) <= leading_lines_to_render + 1 {
                // "join" together two code snippets //
                leading_lines_to_render =
                    (hint.lines.start - prev_hint.lines.end).saturating_sub(1);

                self.output += " |\n";
            } else {
                self.output += "...\n";
            }
        }

        for (i, line) in self
            .code
            .lines()
            .enumerate()
            .map(|(i, l)| (i + 1, l))
            .take(hint.lines.end)
            .skip(hint.lines.start.saturating_sub(leading_lines_to_render + 1))
        {
            // Print line number, line, and gutter //
            let padding = self.padding - i.ilog10();

            self.output += "\x1b[1m";

            for _ in 0..padding {
                self.output += " ";
            }

            self.output += &i.to_string();
            self.output += " | \x1b[m";
            self.output += line;
            self.output += "\n";

            if !(hint.lines.start..=hint.lines.end).contains(&i) || line.is_empty() {
                continue;
            }

            // Print pointer gutter //

            for _ in 0..=self.padding {
                self.output += " ";
            }

            self.output += "\x1b[1m | ";
            self.output += hint.hint.escape;

            // Print pointer

            let arrow_st = if i == hint.lines.start {
                hint.columns.start
            } else {
                1
            };
            let arrow_end = if i == hint.lines.end {
                hint.columns.end
            } else {
                line.chars().count()
            };

            for _ in 0..arrow_st - 1 {
                self.output += " ";
            }

            for _ in arrow_st..=arrow_end.max(arrow_st) {
                self.output.push(hint.pointer_char);
            }

            self.output += "\x1b[m\n";
        }

        if !hint.msg.is_empty() {
            // Print message //
            for _ in 0..=self.padding {
                self.output += " ";
            }
            self.output += "\x1b[1m | ";
            self.output += hint.hint.escape;
            self.output += &hint.msg;
        }

        self.output.push_str("\x1b[m\n");
    }

    pub fn finish(mut self) -> String {
        match &*self.backlog {
            [] => {}
            &[hint] => {
                self.direct_render_hint(None, hint, None);
            }
            &[prev_hint, hint] => {
                self.direct_render_hint(Some(prev_hint), hint, None);
            }
            _ => unreachable!(),
        }

        self.output.push('\n');
        self.output
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

    fn resolve<'a>(&'a self, code: &str) -> ResolvedHintRef<'a> {
        let (line_st, col_st) = util::line_and_col(code, self.span.start);
        let (line_end, col_end) =
            util::line_and_col(code, self.span.end.saturating_sub(1).max(self.span.start));

        ResolvedHintRef {
            hint: self,
            lines: (line_st..line_end).into(),
            columns: (col_st..col_end).into(),
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
