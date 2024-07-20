#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::manual_assert)]
#![allow(clippy::ref_as_ptr)]
#![allow(clippy::type_complexity)]
#![allow(clippy::similar_names)]
#![forbid(clippy::explicit_deref_methods)]
#![forbid(clippy::range_plus_one)]
#![forbid(clippy::semicolon_if_nothing_returned)]
#![forbid(clippy::map_unwrap_or)]
#![forbid(clippy::uninlined_format_args)]

use std::{io::Write as _, process};

use codegen::CodegenContext;
use error_handling::WLangError;
use lexer::{Lexer, LexerError};
use util::MemoryStore;

use crate::{error_handling::Spanned, lexer::Token};

mod lexer;

mod codegen;

#[macro_use]
mod error_handling;

mod util;

mod cmdline;
mod parser;

/* TODO list
 *  - Enforce function visibility in name-store
 *  - Allow functions inside of code blocks
 *  - Use function-based errors for parser
 *  - Add debug information
 */

#[allow(clippy::needless_pass_by_value)]
fn handle_io_error<T>(err: std::io::Error) -> T {
    eprintln!("wlab: {err:?}");
    process::exit(err.raw_os_error().unwrap_or(1))
}

fn main() {
    let params = cmdline::Parameters::args().unwrap();

    if params.input_files.is_empty() {
        eprintln!("wlab: at-least one input file must be specified");
        process::exit(1)
    }

    let do_codegen_phase = params.generate_ir || params.generate_asm || params.generate_object;
    let do_parse_phase = do_codegen_phase || params.generate_ast;
    let do_lex_phase = do_parse_phase || params.lex_files;

    if !do_lex_phase {
        process::exit(0)
    }

    std::fs::create_dir_all(&*params.out_dir).unwrap();

    let context = inkwell::context::Context::create();
    let mut codegen_context = CodegenContext::new(&context);

    let src_store = MemoryStore::new();
    let mut crates = Vec::new();

    for file_name in &params.input_files {
        let source: &str =
            src_store.add(String::from_utf8(std::fs::read(file_name).unwrap()).unwrap());

        // get base file name
        let file_name = file_name.split_terminator('/').last().unwrap();
        let file_name = file_name.strip_suffix(".wlang").unwrap_or(file_name);

        let tokens: Result<Vec<Spanned<Token<'_>>>, LexerError> = Lexer::new(source).collect();

        let tokens = tokens.unwrap_or_else(|err| {
            eprintln!("\n{}", err.render(source));
            process::exit(1)
        });

        if params.lex_files {
            let mut lex_file = std::fs::File::create(format!("{}/{file_name}.lex", params.out_dir))
                .unwrap_or_else(handle_io_error);

            writeln!(lex_file, "{tokens:?}").unwrap_or_else(handle_io_error);
        }

        if !do_parse_phase {
            continue;
        }

        let ast = parser::parse_module(&tokens).unwrap_or_else(|err| {
            eprintln!("\n{}", err.render(source));
            process::exit(1);
        });

        if params.generate_ast {
            let mut ast_file = std::fs::File::create(format!("{}/{file_name}.ast", params.out_dir))
                .unwrap_or_else(handle_io_error);

            writeln!(ast_file, "{ast:?}").unwrap_or_else(handle_io_error);
        }

        if !do_codegen_phase {
            continue;
        }

        let crate_ = codegen_context.create_crate(&ast).unwrap_or_else(|err| {
            eprintln!("\n{}", err.render(source));
            process::exit(1);
        });

        crates.push((source, ast, crate_));
    }

    if !do_codegen_phase {
        return;
    }

    for (source, ast, crate_) in crates {
        codegen_context
            .generate_crate(&crate_, &ast, &params)
            .unwrap_or_else(|err| {
                eprintln!("\n{}", err.render(source));
                process::exit(1);
            });
    }
}
