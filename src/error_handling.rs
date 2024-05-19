use core::str;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut, RangeBounds};
use std::{borrow::Cow, ops::Range};

use crate::util;

pub trait WLangError: Sized {
    fn get_msg(error: &Spanned<Self>, code: &str) -> Cow<'static, str>;
}

/// Includes information about where something appears in a source file
#[derive(Debug)]
pub struct Spanned<T>(pub T, pub Range<usize>);

impl<T> Spanned<T>
where
    T: WLangError,
{
    pub fn render(&self, code: &str) -> String {
        let err_msg = WLangError::get_msg(&self, code);

        let (line_st, col_st) = util::line_and_col(code, self.1.start);
        let (line_end, col_end) = util::line_and_col(code, self.1.end);

        let mut ret_val = "\n".to_owned();

        for (i, line) in code
            .lines()
            .enumerate()
            .take(line_end)
            .skip(line_st.saturating_sub(3))
        {
            // Print line number, line, and gutter //
            let padding = line_end.ilog10() - (i + 1).ilog10();

            for _ in 0..padding {
                ret_val += " ";
            }

            ret_val += &(i + 1).to_string();
            ret_val += " | ";
            ret_val += line;
            ret_val += "\n";

            if !(line_st..=line_end).contains(&(i + 1)) || line.is_empty() {
                continue;
            }

            // Print pointer //

            for _ in 0..line_end.ilog10() + 1 {
                ret_val += " ";
            }

            ret_val += " | ";

            let arrow_st = if i + 1 == line_st { col_st } else { 1 };
            let arrow_end = if i + 1 == line_end {
                col_end
            } else {
                line.chars().count() + 1
            };

            for _ in 0..arrow_st - 1 {
                ret_val += " ";
            }

            for _ in arrow_st..arrow_end {
                ret_val += "^"
            }

            ret_val += "\n";
        }

        for _ in 0..line_end.ilog10() + 1 {
            ret_val += " ";
        }
        ret_val += " | ";
        ret_val += &err_msg;

        return ret_val;
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
