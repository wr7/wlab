use lexer::{Lexer, LexerError};

use crate::lexer::{BracketType, Token};

mod lexer;

mod parser;

fn main() {
    let test_str = r"
    fn foo() {
        biz + bang - fuzz;
        zip - bar;
        zing + {h-p; k}
    }
    
    fn do_thing() {
        fn do_other_thing() {}
        hello;
        do_thing + f
    }";

    let tokens: Result<Vec<Token>, LexerError> = Lexer::new(test_str).collect();
    let tokens = tokens.unwrap();

    let ast = parser::parse_expr_list(&tokens);
    dbg!(ast);
}
