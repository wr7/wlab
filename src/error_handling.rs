use core::str;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::{borrow::Cow, ops::Range};

use crate::util;

pub trait WLangError: Sized {
    fn get_msg(error: &Spanned<Self>, code: &str) -> Cow<'static, str>;
}

#[derive(Debug)]
pub struct Spanned<T>(pub T, pub Range<usize>);

impl<T> Spanned<T>
where
    T: WLangError,
{
    pub fn render(&self, code: &str) -> String {
        let err_msg = WLangError::get_msg(&self, code);
        let col_no = util::column_number(code, self.1.start);
        let line_no = util::line_number(code, self.1.start);

        let mut ret_val = "\n".to_owned();

        for (i, line) in code
            .lines()
            .enumerate()
            .take(line_no)
            .skip(line_no.saturating_sub(3))
        {
            let padding = line_no.ilog10() - (i + 1).ilog10();

            for _ in 0..padding {
                ret_val += " ";
            }

            ret_val += &(i + 1).to_string();
            ret_val += " | ";
            ret_val += line;
            ret_val += "\n";
        }

        for _ in 0..col_no + 3 + line_no.ilog10() as usize {
            ret_val += " ";
        }

        let arrow_width = code
            .get(self.1.clone())
            .map(|c| c.chars().count())
            .unwrap_or(1)
            .max(1);

        for _ in 0..arrow_width {
            ret_val += "^";
        }

        return format!("{ret_val}\n{err_msg} ({line_no}:{col_no})");
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
