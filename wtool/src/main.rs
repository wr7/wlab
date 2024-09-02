use std::{
    io::Read as _,
    path::PathBuf,
    process::{Command, ExitCode, Stdio},
};

use test::TestError;

mod cmdline;

mod test;

fn main() -> ExitCode {
    let arguments = cmdline::Arguments::parse();

    match arguments.sub_args {
        cmdline::SubcommandArguments::Test(test_args) => test::run_tests(test_args)
            .map_err(|err| err.with_prefix("\x1b[0;1;31mwlab test: \x1b[m"))
            .unwrap_or_else(|err| {
                eprintln!("{}", err.text);
                err.code
            }),
        cmdline::SubcommandArguments::Bless => todo!(),
    }
}

/// Builds the compiler and returns the path to it.
fn build_compiler() -> Result<PathBuf, TestError> {
    eprintln!("\n\x1b[1mwtool:\x1b[m building compiler:\n");
    let mut cargo = Command::new("./build_with_path.sh")
        .stdout(Stdio::piped())
        .spawn()?;

    let exit_status = cargo.wait()?;
    let mut stdout = cargo.stdout.take().unwrap();

    let exit_code = exit_status
        .code()
        .ok_or_else(|| TestError::new("cargo build interrupted"))?;

    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    let mut path = Vec::new();
    stdout.read_to_end(&mut path)?;

    eprintln!();

    Ok(String::from_utf8(path)?.into())
}
