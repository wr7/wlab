use core::str;
use std::fmt::Debug;
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
            ret_val += &(i + 1).to_string();
            ret_val += " | ";
            ret_val += line.get(0..80).unwrap_or(line);
            ret_val += "\n";
        }

        ret_val += "    ";

        for _ in 1..col_no {
            ret_val += "~";
        }
        ret_val += "^";

        return format!("{ret_val}\n{err_msg} ({line_no}:{col_no})");
    }
}
