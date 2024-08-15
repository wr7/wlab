use crate::{
    error_handling::{Diagnostic, Spanned as S},
    lexer::Token,
};

pub mod ast;

mod error;
mod macros;
mod rules;
mod util;

pub type TokenStream<'src> = [S<Token<'src>>];

pub fn parse_module<'src>(mut tokens: &TokenStream<'src>) -> Result<ast::Module<'src>, Diagnostic> {
    error::check_brackets(tokens)?;

    let attributes;
    if let Some((attributes_, tokens_)) = rules::try_parse_outer_attributes_from_front(tokens)? {
        tokens = tokens_;
        attributes = attributes_;
    } else {
        attributes = Vec::new();
    }

    let statements = rules::parse_statement_list(tokens)?;

    let mut functions = Vec::new();
    let mut structs = Vec::new();

    for statement in statements {
        let span = statement.1;
        match statement.0 {
            ast::Statement::Function(func) => functions.push(S(func, span)),
            ast::Statement::Struct(struct_) => structs.push(S(struct_, span)),
            _ => return Err(error::expected_function_or_struct(span)),
        }
    }

    Ok(ast::Module {
        attributes,
        functions,
        structs,
    })
}
