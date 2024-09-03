use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

use parse::parse_test;

use crate::{build_compiler, cmdline::test::TestArguments};

mod error;
mod parse;
pub use error::TestError;

/// Runs a series of tests; returns the failing tests
pub fn run_tests(test_args: TestArguments) -> Result<ExitCode, TestError> {
    let test_list;

    match test_args.target {
        crate::cmdline::test::TestTarget::All => todo!(),
        crate::cmdline::test::TestTarget::Tests(tests) => test_list = tests,
    }

    let compiler = build_compiler()?;
    let mut failed_tests = Vec::new();

    for test_name in test_list {
        let mut test_name_buf = PathBuf::from(test_name);

        let test = parse_test(&mut test_name_buf)?;

        let succeeded = run_test(&compiler, &test)?;
        if !succeeded {
            failed_tests.push(test.name)
        }
    }

    Ok(ExitCode::SUCCESS)
}

/// Runs a test. Returns `Ok(false)` if it fails
fn run_test(compiler: &Path, test: &parse::Test) -> Result<bool, TestError> {
    if !compile_test(&test, compiler)? {
        return Ok(false);
    }

    if test.should_fail {
        return Ok(true);
    }

    link_test(&test.name)?;

    let mut test_process = Command::new("./compiler_output/a.out")
        .stdout(Stdio::piped())
        .spawn()
        .and_then(|mut test| {
            test.wait()?;
            Ok(test)
        })
        .map_err(|err| {
            TestError::prefixed(
                err,
                &format!("failed to run test \x1b[m`\x1b[1m{}\x1b[m`: ", &test.name),
            )
        })?;

    let mut test_stdout = Vec::new();
    test_process
        .stdout
        .take()
        .unwrap()
        .read_to_end(&mut test_stdout)
        .unwrap();

    if let Some(expected_stdout) = &test.stdout {
        if expected_stdout != &test_stdout {
            eprintln!(
                "\x1b[1;31wtool test: test `\x1b[m{}\x1b1;31[m` failed: incorrect stdout\x1b[m ",
                &test.name
            );
            eprintln!(
                "\nExpected stdout:\n------------------------------------------\n{}\n------------------------------------------", expected_stdout.escape_ascii()
            );
            eprintln!(
                "\nGot stdout:\n------------------------------------------\n{}\n------------------------------------------", test_stdout.escape_ascii()
            );
            return Ok(false);
        }
    }

    Ok(true)
}

fn compile_test(test: &parse::Test, compiler: &Path) -> Result<bool, TestError> {
    clean()?;

    eprintln!("\x1b[1mwtool:\x1b[m running test `{}`", &test.name);

    let mut compiler = Command::new(compiler)
        .args(&test.args)
        .arg("--")
        .args(&test.sources)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            TestError::prefixed(
                err,
                &format!(
                    "failed to invoke compiler on test \x1b[m`\x1b[1m{}\x1b[m`: ",
                    &test.name
                ),
            )
        })?;

    let exit_status = compiler.wait().unwrap();
    let mut compiler_stderr = Vec::new();
    compiler
        .stderr
        .take()
        .unwrap()
        .read_to_end(&mut compiler_stderr)
        .unwrap();
    if test.should_fail {
        if exit_status.success() {
            eprintln!("\x1b[1;31mwtool test: test `\x1b[m{}\x1b[1;31m` failed:\x1b[m compiler did not fail", &test.name);
            return Ok(false);
        }
    } else {
        if !exit_status.success() {
            eprintln!(
                "\x1b[1;31mwtool test: test `\x1b[m{}\x1b[1;31m` failed:\x1b[m compiler failed",
                &test.name
            );
            eprintln!("\nstderr:\n------------------------------------------");
            std::io::stderr().write_all(&compiler_stderr).unwrap();
            eprintln!("------------------------------------------");

            return Ok(false);
        }
    }
    if let Some(expected_stderr) = &test.stderr {
        if expected_stderr != &compiler_stderr {
            eprintln!(
                "\x1b[1;31wtool test: test `\x1b[m{}\x1b1;31[m` failed: incorrect stderr\x1b[m ",
                &test.name
            );
            eprintln!(
                "\nExpected stderr:\n------------------------------------------\n{}\n------------------------------------------", expected_stderr.escape_ascii()
            );
            eprintln!(
                "\nGot stderr:\n------------------------------------------\n{}\n------------------------------------------", compiler_stderr.escape_ascii()
            );
            return Ok(false);
        }
    }

    Ok(true)
}

fn clean() -> Result<(), TestError> {
    Command::new("./clean.sh")
        .spawn()
        .and_then(|mut c| c.wait())
        .map_err(|err| TestError::prefixed(err, "failed to clean `compiler_output`: "))?;
    Ok(())
}

fn link_test(test: &str) -> Result<(), TestError> {
    let linker_result = Command::new("./link.sh")
        .spawn()
        .and_then(|mut c| c.wait())
        .map_err(|err| {
            TestError::prefixed(err, &format!("failed to run linker for test `{test}`: "))
        })?;

    if !linker_result.success() {
        return Err(TestError::new(format!(
            "linker failed to run for test {test}"
        )));
    }

    Ok(())
}
