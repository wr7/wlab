use std::{io, process::ExitCode, string};

pub struct TestError {
    pub code: ExitCode,
    pub text: String,
}

impl TestError {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            code: ExitCode::FAILURE,
            text: text.into(),
        }
    }

    pub fn prefixed(err: impl Into<Self>, prefix: &str) -> Self {
        err.into().with_prefix(prefix)
    }

    pub fn with_prefix(mut self, prefix: &str) -> TestError {
        self.text.insert_str(0, prefix);
        self
    }
}

impl From<io::Error> for TestError {
    fn from(value: io::Error) -> Self {
        let code = value
            .raw_os_error()
            .and_then(|code| u8::try_from(code).ok())
            .map_or(ExitCode::FAILURE, ExitCode::from);

        let text = value.to_string();

        TestError { code, text }
    }
}

impl From<string::FromUtf8Error> for TestError {
    fn from(value: string::FromUtf8Error) -> Self {
        let text = value.to_string();

        TestError {
            code: ExitCode::FAILURE,
            text,
        }
    }
}

impl From<toml::de::Error> for TestError {
    fn from(value: toml::de::Error) -> Self {
        let text = value.to_string();

        TestError {
            code: ExitCode::FAILURE,
            text,
        }
    }
}
