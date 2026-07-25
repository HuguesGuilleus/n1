use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorKind {
    SubIO,
    NotFound,
    BadRequest,
    Internal,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Error {
    pub kind: ErrorKind,
    pub msg: &'static str,
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            io::ErrorKind::NotFound => NOT_FOUND,
            _ => IO_ERROR,
        }
    }
}

macro_rules! E {
    ($n:ident,$k:ident, $m:expr) => {
        pub const $n: Error = Error {
            kind: ErrorKind::$k,
            msg: $m,
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

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        MUTEX_POISONING
    }
}
