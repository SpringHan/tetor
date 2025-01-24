// App Error

use std::convert::From;

#[derive(Debug, Clone)]
pub struct AppError {
    errors: Vec<ErrorType>
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    IO(std::io::ErrorKind, String),
    Specific(String),
    InvalidCommand(String)
}

impl AppError {
    pub fn new() -> Self {
        AppError { errors: Vec::new() }
    }

    pub fn add_error(&mut self, error: ErrorType) {
        self.errors.push(error);
    }

    pub fn append_errors<I>(&mut self, iter: I)
    where I: Iterator<Item = ErrorType>
    {
        self.errors.extend(iter);
    }

    pub fn clear(&mut self) {
        self.errors.clear();
    }
}

impl ErrorType {
    fn value(&self) -> String {
        match self {
            ErrorType::IO(ref error_kind, ref cause) => {
                format!("[IO Error]: {}\nCause: {}", error_kind.to_string(), cause.to_owned())
            },
            ErrorType::Specific(ref msg) => {
                format!("[Error]: {}!", msg.to_owned())
            },
            ErrorType::InvalidCommand(ref cmd) => {
                format!("[Error]: Invalid Command: {}!", cmd.to_owned())
            },
        }
    }
}

impl From<std::io::Error> for ErrorType {
    fn from(value: std::io::Error) -> Self {
        ErrorType::IO(value.kind(), value.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError { errors: vec![value.into()] }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut msg = String::new();

        for err in self.errors.iter() {
            msg.push_str(&err.value());
            msg.push('\n');
        }

        write!(f, "{}", msg)
    }
}
