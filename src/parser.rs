use crate::{error_handling::Spanned as S, lexer::Token};

pub mod ast;

mod error;
mod macros;
mod rules;
mod util;

pub use error::ParseError;

pub type TokenStream<'src> = [S<Token<'src>>];

pub fn parse_module<'src>(mut tokens: &TokenStream<'src>) -> Result<ast::Module<'src>, ParseError> {
    error::check_brackets(tokens)?;

    let attributes;
    if let Some((attributes_, tokens_)) = rules::try_parse_outer_attributes_from_front(tokens)? {
        tokens = tokens_;
        attributes = attributes_;
    } else {
        attributes = Vec::new();
    }

    let statements = rules::parse_statement_list(tokens)?;
    let functions: Result<Vec<S<ast::Function>>, _> = statements
        .into_iter()
        .map(|S(statement, span)| {
            ast::Function::try_from(statement)
                .map(|s| S(s, span))
                .map_err(|()| ParseError::ExpectedFunction(span))
        })
        .collect();

    let functions = functions?;

    Ok(ast::Module {
        attributes,
        functions,
    })
}
