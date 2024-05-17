use lexer::{Lexer, LexerError};

use crate::lexer::{BracketType, Token};

mod lexer;

mod parser;

fn main() {
    let test_str = r"
    fn foo() {
        biz + bang - fuzz
    }";

    let tokens: Result<Vec<Token>, LexerError> = Lexer::new(test_str).collect();
    let tokens = tokens.unwrap();

    let ast = parser::try_parse_expr(&tokens).unwrap();
    dbg!(ast);
}
