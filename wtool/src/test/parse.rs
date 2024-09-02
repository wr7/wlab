use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use super::TestError;

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Test {
    pub name: String,

    #[serde(skip)]
    pub stdout: Option<Vec<u8>>,

    #[serde(skip)]
    pub stderr: Option<Vec<u8>>,

    #[serde(default)]
    pub should_fail: bool,

    #[serde(default = "default_sources")]
    pub sources: Vec<PathBuf>,

    #[serde(default = "default_args")]
    pub args: Vec<String>,
}

fn default_sources() -> Vec<PathBuf> {
    vec!["test.wlang".into()]
}

fn default_args() -> Vec<String> {
    vec![
        "--output-dir=compiler_output".to_owned(),
        "std/std.wlang".to_owned(),
    ]
}

pub fn parse_test(path: &mut PathBuf) -> Result<Test, TestError> {
    path.push("test.toml");
    let mut test = parse_toml(&path)?;
    path.pop();

    path.push("stdout");
    test.stdout = read_file(&path)?;
    path.pop();

    path.push("stderr");
    test.stderr = read_file(&path)?;
    path.pop();

    // Get full path to file sources //
    {
        let mut tmp_buf = PathBuf::new();

        for source in &mut test.sources {
            tmp_buf.clear();
            tmp_buf.push("./tests");
            tmp_buf.push(&test.name);
            tmp_buf.push(&*source);

            std::mem::swap(&mut tmp_buf, source);
        }
    }

    Ok(test)
}

fn parse_toml(path: &Path) -> Result<Test, TestError> {
    let file = read_file(path)?.ok_or_else(|| {
        TestError::new(format!(
            "could not find `test.toml` file `{}`",
            path.display()
        ))
    })?;

    let file = String::from_utf8(file).map_err(|e| {
        TestError::prefixed(e, &format!("error while reading file `{}`", path.display()))
    })?;

    let test: Test = toml::from_str(&file).map_err(|e| {
        TestError::prefixed(e, &format!("error while parsing file `{}`", path.display()))
    })?;

    Ok(test)
}

/// Reads a file; returns `Ok(None)` if it does not exists.
pub fn read_file(path: &Path) -> Result<Option<Vec<u8>>, TestError> {
    match std::fs::read(&*path) {
        Ok(file) => Ok(Some(file)),
        Err(err) => {
            if err.kind() == ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(TestError::prefixed(
                    err,
                    &format!("failed to open `{}`: ", path.display()),
                ))
            }
        }
    }
}
