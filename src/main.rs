use lexer::{Lexer, LexerError};

use crate::{error_handling::Spanned, lexer::Token};

mod lexer;

mod error_handling;
mod util;

mod parser;

/* TODO
 *  - Allow multiple spans in errors, and fix multi-line spans
 *  - Add more error handling (including for binary operators)
 *  - Test `let bang = (foo) hello; bang = x` to see if it properly triggers an error.
 */

fn main() {
    let test_str = "\
fn foo() {
    fn biz() {
        hello;
    }



    foo + bar;

    let y = f; hello world;
    let bang = do_thing;
    let foo = thing - other_thing;
    let bar = hello + _pog * foo;
    foo = b + c*d;
}
";

    let tokens: Result<Vec<Spanned<Token>>, Spanned<LexerError>> = Lexer::new(test_str).collect();
    let tokens = match tokens {
        Ok(tokens) => tokens,
        Err(err) => panic!("lexer error: {}", err.render(test_str)),
    };

    let ast = parser::parse(&tokens);
    let ast = match ast {
        Ok(ast) => ast,
        Err(e) => panic!("parse error: {}", e.render(test_str)),
    };

    dbg!(ast);
}
