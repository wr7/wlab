#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::manual_assert)]
#![allow(clippy::ref_as_ptr)]
#![forbid(clippy::explicit_deref_methods)]
#![forbid(clippy::range_plus_one)]
#![forbid(clippy::semicolon_if_nothing_returned)]
#![forbid(clippy::map_unwrap_or)]
#![forbid(clippy::uninlined_format_args)]

use std::{io::Write as _, process};

use error_handling::WLangError;
use lexer::{Lexer, LexerError};

use crate::{error_handling::Spanned, lexer::Token};

mod lexer;

mod codegen;

#[macro_use]
mod error_handling;

mod util;

mod parser;

/* Long term TODO list
 *  - Add function return values and compound expressions
 *  - Add function definition to errors
 *  - Add a "function already defined" error
 *  - Allow out-of-order and recursive functions
 *  - Add unit tests for parser
 */

fn main() {
    let input: &str = &String::from_utf8(std::fs::read("./a.wlang").unwrap()).unwrap();

    let tokens: Result<Vec<Spanned<Token>>, LexerError> = Lexer::new(input).collect();

    let tokens = match tokens {
        Ok(tokens) => tokens,
        Err(err) => {
            eprintln!("\n{}", err.render(input));
            process::exit(1);
        }
    };

    let mut lex_file = std::fs::File::create("./a.lex").unwrap();
    write!(&mut lex_file, "{tokens:#?}").unwrap();

    let ast = parser::parse(&tokens);
    let ast = match ast {
        Ok(ast) => ast,
        Err(err) => {
            eprintln!("\n{}", err.render(input));
            process::exit(1);
        }
    };

    let mut ast_file = std::fs::File::create("./a.ast").unwrap();
    write!(&mut ast_file, "{ast:#?}").unwrap();

    if let Err(err) = codegen::generate_code(&ast) {
        eprintln!("\n{}", err.render(input));
        process::exit(1);
    };
}
