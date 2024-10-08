use crate::{error_handling::Spanned, lexer::Token};

mod split;

pub use split::*;

/// Iterates over tokens that are not surrounded by brackets.
#[derive(Clone)]
pub(super) struct NonBracketedIter<'a, 'src> {
    remaining: &'a [Spanned<Token<'src>>],
    bracket_level_start: usize,
    bracket_level_end: usize,
}

impl<'a, 'src> NonBracketedIter<'a, 'src> {
    pub fn new(slc: &'a [Spanned<Token<'src>>]) -> Self {
        Self {
            remaining: slc,
            bracket_level_start: 0,
            bracket_level_end: 0,
        }
    }

    pub fn remainder<'b>(&'b self) -> &'a [Spanned<Token<'src>>] {
        self.remaining
    }
}

impl<'a, 'src> Iterator for NonBracketedIter<'a, 'src> {
    type Item = &'a Spanned<Token<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some((token, remaining)) = self.remaining.split_first() else {
                assert_eq!(self.bracket_level_start, self.bracket_level_end);
                return None;
            };
            self.remaining = remaining;

            match &**token {
                Token::OpenBracket(_) => self.bracket_level_start += 1,
                Token::CloseBracket(_) => self.bracket_level_start -= 1,
                _ => {}
            }

            if self.bracket_level_start == 0 {
                return Some(token);
            }

            if self.bracket_level_start == 1 && matches!(&**token, Token::OpenBracket(_)) {
                return Some(token);
            }
        }
    }
}

impl<'a, 'src> DoubleEndedIterator for NonBracketedIter<'a, 'src> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            let Some((token, remaining)) = self.remaining.split_last() else {
                assert_eq!(self.bracket_level_start, self.bracket_level_end);
                return None;
            };
            self.remaining = remaining;

            match &**token {
                Token::OpenBracket(_) => self.bracket_level_end -= 1,
                Token::CloseBracket(_) => self.bracket_level_end += 1,
                _ => {}
            }

            if self.bracket_level_end == 0 {
                return Some(token);
            }

            if self.bracket_level_end == 1 && matches!(&**token, Token::CloseBracket(_)) {
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
        let src = "9 + ( {10 - 5}; - 2 ) = 21";

        let tokens = Lexer::new(src)
            .collect::<Result<Vec<Spanned<Token>>, _>>()
            .unwrap_or_else(|err| {
                panic!("{}", err.render(src));
            });

        let mut iter = NonBracketedIter::new(&tokens);
        assert_eq!(&**iter.next().unwrap(), &T!("9"));
        assert_eq!(&**iter.next().unwrap(), &T!("+"));
        assert_eq!(&**iter.next().unwrap(), &T!("("));
        assert_eq!(&**iter.next().unwrap(), &T!(")"));
        assert_eq!(&**iter.next().unwrap(), &T!("="));
        assert_eq!(&**iter.next().unwrap(), &T!("21"));
        assert_eq!(iter.next(), None);
    }
}
