use wutil::Span;

use crate::{error_handling::Spanned, lexer::Token, parser::util::NonBracketedIter};

/// Splits by tokens that patch a predicate. This takes brackets into consideration.
pub struct TokenSplit<'a, 'src, P>
where
    P: FnMut(&'a Token<'src>) -> bool,
{
    tokens: &'a [Spanned<Token<'src>>],
    nb_iter: Option<NonBracketedIter<'a, 'src>>,
    predicate: P,
}

impl<'a, 'src, P> TokenSplit<'a, 'src, P>
where
    P: FnMut(&'a Token<'src>) -> bool,
{
    pub fn new(tokens: &'a [Spanned<Token<'src>>], predicate: P) -> Self {
        Self {
            tokens,
            nb_iter: Some(NonBracketedIter::new(tokens)),
            predicate,
        }
    }
}

impl<'a, 'src, P> Iterator for TokenSplit<'a, 'src, P>
where
    P: FnMut(&'a Token<'src>) -> bool,
{
    type Item = (&'a [Spanned<Token<'src>>], Option<&'a Spanned<Token<'src>>>);

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
