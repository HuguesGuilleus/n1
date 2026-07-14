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
