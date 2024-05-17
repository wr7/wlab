use lexer::{Lexer, LexerError};

use crate::lexer::{BracketType, Token};

mod lexer;

mod parser;

fn main() {
    let test_str = r"
    fn foo() {
        let foo = thing + other_thing;
        foo = b + c*d;
    }
";

    let tokens: Result<Vec<Token>, LexerError> = Lexer::new(test_str).collect();
    let tokens = tokens.unwrap();

    let ast = parser::parse_statement_list(&tokens);
    dbg!(ast);
}
