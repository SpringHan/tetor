// App Error

use std::convert::From;

pub type AppResult<T> = Result<T, AppError>;

// TODO: Seems that app error in this package not needs to be a vec
#[derive(Debug, Clone, Default)]
pub struct AppError {
    errors: Vec<ErrorType>
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    IO(tokio::io::ErrorKind, String),
    Specific(String),
}

impl AppError {
    pub fn empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn into_iter(self) -> impl Iterator<Item = ErrorType> {
        self.errors.into_iter()
    }

    pub fn append_errors<I>(&mut self, iter: I)
    where I: Iterator<Item = ErrorType>
    {
        self.errors.extend(iter);
    }

    pub fn get_first(&self) -> String {
        self.errors.first()
            .expect("Error code 1 at pop in error_type.rs!")
            .value()
    }

    pub fn throw(&mut self) {
        self.errors.remove(0);
    }
}

impl ErrorType {
    pub fn pack(self) -> AppError {
        AppError { errors: vec![self] }
    }

    fn value(&self) -> String {
        match self {
            ErrorType::IO(ref error_kind, ref cause) => {
                format!("[IO Error]: {}\nCause: {}", error_kind.to_string(), cause.to_owned())
            },
            ErrorType::Specific(ref msg) => {
                format!("[Error]: {}!", msg.to_owned())
            }
        }
    }
}

impl From<tokio::io::Error> for ErrorType {
    fn from(value: std::io::Error) -> Self {
        ErrorType::IO(value.kind(), value.to_string())
    }
}

impl From<tokio::io::Error> for AppError {
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

#[macro_export]
macro_rules! panic_error {
    () => {
        return Err(crate::error::ErrorType::UnknowPanicError.pack())
    };
}
