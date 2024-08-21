use crate::{
    error_handling::Hint,
    util::{self, MaxVec},
};

use std::ops::Deref;

use wutil::Span;

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

impl Hint {
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

pub(super) struct DiagnosticRenderer<'a> {
    code: &'a str,
    output: String,
    padding: u32,
    backlog: MaxVec<ResolvedHintRef<'a>, 2>,
}

impl<'a> DiagnosticRenderer<'a> {
    pub fn new(code: &'a str, hints: &[Hint], diagnostic_msg: &str) -> Self {
        pub(crate) fn get_padding(code: &str, hints: &[Hint]) -> u32 {
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

    pub fn render_hint(&mut self, hint: &'a Hint) {
        let next_hint = hint.resolve(self.code);
        let prev_hint = self.backlog.cycle(next_hint);
        let hint = self.backlog[0];

        if self.backlog.len() <= 1 {
            return;
        }

        self.direct_render_hint(prev_hint, hint, Some(next_hint));
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
}
