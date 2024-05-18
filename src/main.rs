use std::process::abort;

use lexer::{Lexer, LexerError};

use crate::{error_handling::Spanned, lexer::Token};

mod lexer;

mod error_handling;
mod util;

mod parser;

fn main() {
    let test_str = "\
    fn foo() {
        fn biz() {
            hello
        }
        let bang = do_thing;
        let foo = thing - other_thing;
        let bar = hello + _pog * foo;
        foo = b + c*d;
    }
";

    let tokens: Result<Vec<Token>, Spanned<LexerError>> =
        Lexer::new(test_str).map(|t| t.map(|t| t.0)).collect();
    let tokens = match tokens {
        Ok(tokens) => tokens,
        Err(err) => panic!("lexer error: {}", err.render(test_str)),
    };

    let ast = parser::parse(&tokens);
    dbg!(ast);
}
