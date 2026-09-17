use std::{io, time::SystemTimeError};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Clone)]
/// An error with the context.
pub struct Error {
    /// The base of the error.
    pub atomic: AtomicError,
    /// All contextual information about this error.
    /// Can be empty.
    pub context: Vec<String>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AtomicError {
    pub kind: ErrorKind,
    pub message: &'static str,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorKind {
    BadRequest,
    Forbidden,
    Internal,
    NoAuth,
    NotFound,
    SubIO,
}

impl AtomicError {
    pub fn push<S: ToString>(self, msg: S) -> Result<()> {
        Err(Error {
            atomic: self,
            context: vec![msg.to_string()],
        })
    }
}

impl Error {
    /// Add new contextual information.
    pub fn push(mut self, info: String) -> Self {
        self.context.push(info);
        self
    }
}

impl From<AtomicError> for Error {
    fn from(atomic: AtomicError) -> Self {
        Self {
            atomic,
            context: vec![],
        }
    }
}

impl From<io::Error> for AtomicError {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            io::ErrorKind::NotFound => NOT_FOUND,
            _ => IO_ERROR,
        }
    }
}

impl From<SystemTimeError> for AtomicError {
    fn from(_: SystemTimeError) -> Self {
        AtomicError {
            kind: ErrorKind::Internal,
            message: "get system time elapsed",
        }
    }
}

impl<T> From<std::sync::PoisonError<T>> for AtomicError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        MUTEX_POISONING
    }
}

macro_rules! E {
    ($n:ident,$k:ident, $m:expr) => {
        pub const $n: AtomicError = AtomicError {
            kind: ErrorKind::$k,
            message: $m,
        };
    };
}

E!(NOT_FOUND, NotFound, "not found");
E!(IO_ERROR, NotFound, "io operation fail");
E!(MUTEX_POISONING, Internal, "mutex is poisoning");
E!(EOF, Internal, "end of file");
E!(TIME_FAIL, Internal, "get time is fall");
E!(DB_ENCODE, Internal, "encode data from DB fail");
E!(DB_DECODE, Internal, "decode data from DB fail");
