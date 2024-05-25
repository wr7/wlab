use error_handling::WLangError;
use lexer::{Lexer, LexerError};

use crate::{error_handling::Spanned, lexer::Token};

mod lexer;

mod codegen;

#[macro_use]
mod error_handling;

mod util;

mod parser;

/* Parser TODO
 *  - Create distinction between Compound Statements and Compound Expressions (maybe have compound expression return implicit unit?)
 *  - Clean up parse_statement_list
 */

fn main() {
    let test_str = "\
fn do_nothing(a) {
    
}

fn main(foo, bar) {
    let x = do_nothing(foo + bar / 42069);
}";

    let tokens: Result<Vec<Spanned<Token>>, LexerError> = Lexer::new(test_str).collect();

    let tokens = match tokens {
        Ok(tokens) => tokens,
        Err(err) => {
            eprintln!("\n{}", err.render(test_str));
            return;
        }
    };

    let ast = parser::parse(&tokens);
    let ast = match ast {
        Ok(ast) => ast,
        Err(err) => {
            eprintln!("\n{}", err.render(test_str));
            return;
        }
    };

    dbg!(&ast);

    if let Err(err) = codegen::generate_code(&ast) {
        eprintln!("\n{}", err.render(test_str));
        return;
    };
}
