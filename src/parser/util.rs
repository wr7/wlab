use std::ops::Deref;

use crate::{error_handling::Spanned, lexer::Token};

mod split;

pub use split::*;

/// Iterates over tokens that are not surrounded by brackets.
/// # Example
/// # use wutil::lexer::Lexer;
/// let tokens = lexer::new
#[derive(Clone)]
pub struct NonBracketedIter<'a> {
    remaining: &'a [Spanned<Token<'a>>],
    bracket_level_start: usize,
    bracket_level_end: usize,
}

impl<'a> NonBracketedIter<'a> {
    pub(super) fn new(slc: &'a [Spanned<Token<'a>>]) -> Self {
        Self {
            remaining: slc,
            bracket_level_start: 0,
            bracket_level_end: 0,
        }
    }
}

impl<'a> Iterator for NonBracketedIter<'a> {
    type Item = &'a Spanned<Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some((token, remaining)) = self.remaining.split_first() else {
                assert_eq!(self.bracket_level_start, self.bracket_level_end);
                return None;
            };
            self.remaining = remaining;

            match token.deref() {
                Token::OpenBracket(_) => self.bracket_level_start += 1,
                Token::CloseBracket(_) => self.bracket_level_start -= 1,
                _ => {}
            }

            if self.bracket_level_start == 0 {
                return Some(token);
            }

            if self.bracket_level_start == 1 && matches!(token.deref(), Token::OpenBracket(_)) {
                return Some(token);
            }
        }
    }
}

impl<'a> DoubleEndedIterator for NonBracketedIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let Some((token, remaining)) = self.remaining.split_last() else {
                assert_eq!(self.bracket_level_start, self.bracket_level_end);
                return None;
            };
            self.remaining = remaining;

            match token.deref() {
                Token::OpenBracket(_) => self.bracket_level_end -= 1,
                Token::CloseBracket(_) => self.bracket_level_end += 1,
                _ => {}
            }

            if self.bracket_level_end == 0 {
                return Some(token);
            }

            if self.bracket_level_end == 1 && matches!(token.deref(), Token::CloseBracket(_)) {
                return Some(token);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, T};

    use super::*;

    #[test]
    fn non_bracketed_test() {
        let tokens: Vec<Spanned<Token>> = Lexer::new("9 + ( {10 - 5}; - 2 ) = 21")
            .collect::<Result<Vec<Spanned<Token>>, _>>()
            .unwrap();

        let mut iter = NonBracketedIter::new(&tokens);
        assert_eq!(iter.next().unwrap().deref(), &T!("9"));
        assert_eq!(iter.next().unwrap().deref(), &T!("+"));
        assert_eq!(iter.next().unwrap().deref(), &T!("("));
        assert_eq!(iter.next().unwrap().deref(), &T!(")"));
        assert_eq!(iter.next().unwrap().deref(), &T!("="));
        assert_eq!(iter.next().unwrap().deref(), &T!("21"));
        assert_eq!(iter.next(), None);
    }
}
