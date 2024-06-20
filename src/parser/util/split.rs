use wutil::Span;

use crate::{error_handling::Spanned, lexer::Token, util::SliceExt};

use super::NonBracketedIter;

/// Splits by tokens that patch a predicate. This takes brackets into consideration.
pub struct TokenSplit<'a, P>
where
    P: FnMut(&'a Token<'a>) -> bool,
{
    tokens: &'a [Spanned<Token<'a>>],
    nb_iter: Option<NonBracketedIter<'a>>,
    predicate: P,
}

impl<'a, P> TokenSplit<'a, P>
where
    P: FnMut(&'a Token<'a>) -> bool,
{
    pub fn new(tokens: &'a [Spanned<Token<'a>>], predicate: P) -> Self {
        Self {
            tokens,
            nb_iter: Some(NonBracketedIter::new(tokens)),
            predicate,
        }
    }
}

impl<'a, P> Iterator for TokenSplit<'a, P>
where
    P: FnMut(&'a Token<'a>) -> bool,
{
    type Item = (&'a [Spanned<Token<'a>>], Option<&'a Spanned<Token<'a>>>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut span: Span = (self.tokens.len()..self.tokens.len()).into();

        let separator;

        loop {
            let Some(tok) = self.nb_iter.as_mut()?.next() else {
                self.nb_iter = None;
                separator = None;
                break;
            };

            let idx = self.tokens.elem_offset(tok).unwrap();

            span.start = span.start.min(idx);

            if (self.predicate)(tok) {
                separator = Some(tok);
                span.end = idx;
                break;
            }
        }

        Some((&self.tokens[span], separator))
    }
}
