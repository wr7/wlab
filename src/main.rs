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
use error_handling::Diagnostic;
use lexer::Lexer;
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
 *  - Allow functions and structs inside of code blocks
 *  - Add a `Never` type and make `loop` return it
 *       - Add a "dead code" warning
 *  - Structs
 *       - Add visibility
 *       - Fix structs with out-of-order struct fields (depgraph)
 *       - Properly handle recursively-defined types
 *  - Give parser access to source code to further reduce allocations
 *  - Use more efficient representation of ast::Path
 *  - Debug info
 *      - Create DILexicalScope for all code blocks (not just functions)
 *      - Add debug info for variables
 *      - Fix mangled names in debug info
 */

#[allow(clippy::needless_pass_by_value)]
fn handle_io_error<T>(err: std::io::Error) -> T {
    eprintln!("wlab: {err:?}");
    process::exit(err.raw_os_error().unwrap_or(1))
}

fn main() {
    let params = cmdline::Parameters::parse().unwrap_or_else(|err| {
        eprintln!("\x1b[1;31mwlab error: \x1b[m{err}");
        std::process::exit(1)
    });

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

    let context = wllvm::Context::new();
    let mut codegen_context = CodegenContext::new(&context, &params);

    let src_store = MemoryStore::new();
    let mut crates = Vec::new();

    for file_name in &params.input_files {
        let source: &str = src_store.add(
            String::from_utf8(std::fs::read(file_name).unwrap_or_else(|err| {
                eprintln!("wlab: failed to open file `{file_name}`: {err}");
                std::process::exit(1)
            }))
            .unwrap(),
        );

        // get base file name
        let file_base_name = file_name.split_terminator('/').last().unwrap();
        let file_base_name = file_base_name
            .strip_suffix(".wlang")
            .unwrap_or(file_base_name);

        let tokens: Result<Vec<Spanned<Token<'_>>>, Diagnostic> = Lexer::new(source).collect();

        let tokens = tokens.unwrap_or_else(|err| {
            eprintln!("\n{}", err.render(source));
            process::exit(1)
        });

        if params.lex_files {
            let mut lex_file =
                std::fs::File::create(format!("{}/{file_base_name}.lex", params.out_dir))
                    .unwrap_or_else(handle_io_error);

            writeln!(lex_file, "{tokens:?}").unwrap_or_else(handle_io_error);
        }

        if !do_parse_phase {
            continue;
        }

        let ast = parser::parse_module(&tokens).unwrap_or_else(|mut err| {
            err.prepend("Failed to parse file: ");

            eprintln!("\n{}", err.render(source));
            process::exit(1);
        });

        if params.generate_ast {
            let mut ast_file =
                std::fs::File::create(format!("{}/{file_base_name}.ast", params.out_dir))
                    .unwrap_or_else(handle_io_error);

            writeln!(ast_file, "{ast:#?}").unwrap_or_else(handle_io_error);
        }

        if !do_codegen_phase {
            continue;
        }

        let crate_ = codegen_context
            .create_crate(&ast, source, file_name.clone())
            .unwrap_or_else(|err| {
                eprintln!("\n{}", err.render(source));
                process::exit(1);
            });

        crates.push((source, ast, crate_));
    }

    if !do_codegen_phase {
        return;
    }

    for (source, ast, crate_) in &crates {
        codegen_context
            .add_functions(ast, crate_)
            .unwrap_or_else(|err| {
                eprintln!("\n{}", err.render(source));
                process::exit(1);
            });
    }

    for (source, ast, crate_) in &crates {
        codegen_context
            .generate_crate(crate_, ast, &params, source)
            .unwrap_or_else(|err| {
                eprintln!("\n{}", err.render(source));
                process::exit(1);
            });
    }
}
