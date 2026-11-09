use std::{fmt::Display, io, num::ParseIntError, str::Utf8Error};

use n1_tool::DBError::{self, DBInternal};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(pub Vec<ErrorStep>);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorCategory {
    AuthForbidden,
    AuthNone,
    BadRequest,
    Internal,
    NotFound,
}

impl Error {
    pub fn push(mut self, context: impl Into<ErrorStep>) -> Self {
        self.0.push(context.into());
        self
    }

    pub fn push_result(mut self, context: impl Into<ErrorStep>) -> Result<()> {
        self.0.push(context.into());
        Err(self)
    }

    pub fn category(&self) -> ErrorCategory {
        self.0
            .get(0)
            .map(|step| step.category())
            .unwrap_or(ErrorCategory::Internal)
    }
}

impl From<ErrorStep> for Error {
    fn from(value: ErrorStep) -> Self {
        Error(vec![value])
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for step in &self.0 {
            write!(f, "{}: ", step.to_string())?;
        }
        Ok(())
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorStep {
    Action(&'static str),
    AuthForbiddenAdmin,
    AuthForbiddenRead {
        owner: u32,
        app: u16,
    },
    AuthForbiddenWrite {
        owner: u32,
        app: u16,
    },
    AuthNone,
    DB(DBError),
    DBDataNotString {
        owner: u32,
        object: u32,
        err: Utf8Error,
    },
    ExpectRealEntity {
        entity: u32,
    },
    FieldEmpty {
        name: &'static str,
    },
    FieldParsing {
        name: &'static str,
    },
    FieldZero {
        name: &'static str,
    },
    NameEmpty,
    NameInvalid,
    NameTooLong {
        len: usize,
    },
    NeedNoData,
    NotFoundEntity {
        entity: u32,
    },
    NotFoundObject {
        owner: u32,
        object: u32,
        type_name: &'static str,
    },
    Parsing2Integer,
    ParsingInteger {
        err: ParseIntError,
    },
    ReadBody(io::Error),

    X(String),
    Static(&'static str),
}

impl ErrorStep {
    pub fn result(self) -> Result<()> {
        Err(Error(vec![self]))
    }

    pub fn category(&self) -> ErrorCategory {
        match self {
            ErrorStep::AuthNone => ErrorCategory::AuthNone,
            ErrorStep::AuthForbiddenAdmin => ErrorCategory::AuthForbidden,
            ErrorStep::AuthForbiddenRead { .. } => ErrorCategory::AuthForbidden,
            ErrorStep::AuthForbiddenWrite { .. } => ErrorCategory::AuthForbidden,
            ErrorStep::ReadBody(..) => ErrorCategory::BadRequest,
            ErrorStep::Action(..) => ErrorCategory::BadRequest,
            ErrorStep::DB(DBError::ClientConnexion) => ErrorCategory::BadRequest,
            ErrorStep::DB(
                DBError::Decode(..)
                | DBError::Encode(..)
                | DBError::GetTime
                | DBInternal
                | DBError::Io(..)
                | DBError::MagicNumber { .. },
            ) => ErrorCategory::Internal,
            ErrorStep::DB(DBError::NotFoundID { .. } | DBError::NotFoundPath { .. }) => {
                ErrorCategory::NotFound
            }
            ErrorStep::DBDataNotString { .. } => ErrorCategory::Internal,
            ErrorStep::ExpectRealEntity { .. } => ErrorCategory::BadRequest,
            ErrorStep::FieldEmpty { .. } => ErrorCategory::BadRequest,
            ErrorStep::FieldParsing { .. } => ErrorCategory::BadRequest,
            ErrorStep::FieldZero { .. } => ErrorCategory::BadRequest,
            ErrorStep::NameEmpty => ErrorCategory::BadRequest,
            ErrorStep::NameInvalid => ErrorCategory::BadRequest,
            ErrorStep::NameTooLong { .. } => ErrorCategory::BadRequest,
            ErrorStep::NeedNoData => ErrorCategory::BadRequest,
            ErrorStep::NotFoundEntity { .. } => ErrorCategory::NotFound,
            ErrorStep::NotFoundObject { .. } => ErrorCategory::NotFound,
            ErrorStep::Parsing2Integer => ErrorCategory::BadRequest,
            ErrorStep::ParsingInteger { .. } => ErrorCategory::BadRequest,

            ErrorStep::X(..) => ErrorCategory::BadRequest,
            ErrorStep::Static(..) => ErrorCategory::BadRequest,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            ErrorStep::Action(action) => format!("in action {}", action),
            ErrorStep::AuthNone => "need auth".to_string(),
            ErrorStep::AuthForbiddenAdmin => "access forbidden for non-admin".to_string(),
            ErrorStep::AuthForbiddenRead { owner, app } => {
                format!("forbidden read access to {owner} for app {app}")
            }
            ErrorStep::AuthForbiddenWrite { owner, app } => {
                format!("forbidden write access to {owner} for app {app}")
            }
            ErrorStep::ReadBody(err) => format!("read body fail: {}", err),
            ErrorStep::DB(DBError::ClientConnexion) => "client connexion is broken".to_string(),

            ErrorStep::DB(DBError::Decode(err)) => format!("Decode client data: {}", err),
            ErrorStep::DB(DBError::Encode(err)) => format!("Encode client data: {}", err),
            ErrorStep::DB(DBError::GetTime) => format!("get time from system"),
            ErrorStep::DB(DBError::DBInternal) => format!("database internal error"),
            ErrorStep::DB(DBError::Io(kind)) => format!("io error: {:?}", kind),
            ErrorStep::DB(DBError::MagicNumber { got, expected }) => {
                format!("wrong magic number: got {:?}, expected {:?}", got, expected)
            }
            ErrorStep::DB(DBError::NotFoundID { owner, object }) => {
                format!("not found owner={owner} object={object}")
            }
            ErrorStep::DB(DBError::NotFoundPath { path }) => format!("not found path : {:?}", path),
            ErrorStep::DBDataNotString { owner, object, err } => {
                format!("DB data not string: owner={owner} object={object} err={err}")
            }
            ErrorStep::ExpectRealEntity { entity } => format!("expect real entity for {entity}"),
            ErrorStep::FieldZero { name } => format!("field {name:?} is zero"),
            ErrorStep::FieldParsing { name } => format!("parsing fail in field {name:?}"),
            ErrorStep::FieldEmpty { name } => format!("field {name:?} is empty"),
            ErrorStep::NameEmpty => "name could not be empty".to_string(),
            ErrorStep::NameInvalid => "invalid name (contain spaces or underscores)".to_string(),
            ErrorStep::NameTooLong { len } => format!("name too long: {len}"),
            ErrorStep::NeedNoData => "need no data".to_string(),
            ErrorStep::NotFoundEntity { entity } => format!("not found entity {entity}"),
            ErrorStep::NotFoundObject {
                owner,
                object,
                type_name,
            } => format!("not found for owner={owner} object({type_name})={object}"),
            ErrorStep::Parsing2Integer => {
                "need `owner,object` format with two integers".to_string()
            }
            ErrorStep::ParsingInteger { err } => format!("parsing integer: {err}"),

            ErrorStep::X(err) => err.to_string(),
            ErrorStep::Static(err) => err.to_string(),
        }
    }
}

impl From<DBError> for Error {
    fn from(err: DBError) -> Self {
        Error(vec![ErrorStep::DB(err)])
    }
}

// LEGACY ERRORS ZONE

pub struct AtomicError(pub ErrorCategory, pub &'static str);

pub const NOT_FOUND: AtomicError = AtomicError(ErrorCategory::NotFound, "not found");
pub const IO_ERROR: AtomicError = AtomicError(ErrorCategory::Internal, "io operation fail");
pub const IO_MAGIC_NUMBER: AtomicError =
    AtomicError(ErrorCategory::NotFound, "invalid magic number");
pub const MUTEX_POISONING: AtomicError = AtomicError(ErrorCategory::Internal, "mutex is poisoning");
pub const EOF: AtomicError = AtomicError(ErrorCategory::Internal, "end of file");
pub const TIME_FAIL: AtomicError = AtomicError(ErrorCategory::Internal, "get time is fall");
pub const DB_ENCODE: AtomicError = AtomicError(ErrorCategory::Internal, "encode data from DB fail");
pub const DB_DECODE: AtomicError = AtomicError(ErrorCategory::Internal, "decode data from DB fail");

pub const ALREADY_CONNECTED: AtomicError =
    AtomicError(ErrorCategory::AuthForbidden, "already connected");
pub const ALREADY_EXIST: AtomicError = AtomicError(ErrorCategory::BadRequest, "already exist");
pub const DATA_INCOHERENT: AtomicError =
    AtomicError(ErrorCategory::Internal, "data are incoherent");
pub const DECODE_REQUEST: AtomicError =
    AtomicError(ErrorCategory::BadRequest, "cannot decode request data");
pub const EXPECT_REEL_ENTITY: AtomicError =
    AtomicError(ErrorCategory::BadRequest, "expected a user of a group");
pub const EXPECTED: AtomicError = AtomicError(ErrorCategory::BadRequest, "expected something else");
pub const FIELD_CONTENT: AtomicError =
    AtomicError(ErrorCategory::BadRequest, "empty 'content' field");
pub const FIELD_DESC: AtomicError = AtomicError(ErrorCategory::BadRequest, "empty 'desc' field");
pub const FIELD_EMPTY: AtomicError = AtomicError(ErrorCategory::BadRequest, "empty field");
pub const FIELD_ENTITY: AtomicError =
    AtomicError(ErrorCategory::BadRequest, "empty entity ID 'eid' field");
pub const FIELD_ID_ZERO: AtomicError =
    AtomicError(ErrorCategory::BadRequest, "identifier field is zero");
pub const FIELD_NAME: AtomicError = AtomicError(ErrorCategory::BadRequest, "empty 'name' field");
pub const FIELD_OID: AtomicError =
    AtomicError(ErrorCategory::BadRequest, "empty object ID 'oid' field");
pub const FIELD_PASSWORD: AtomicError =
    AtomicError(ErrorCategory::BadRequest, "empty 'password' field");
pub const FIELD_STR: AtomicError = AtomicError(ErrorCategory::BadRequest, "empty 'str' field");
pub const FIELD_TITLE: AtomicError = AtomicError(ErrorCategory::BadRequest, "empty 'title' field");
pub const FORBIDEN_GLOBAL: AtomicError =
    AtomicError(ErrorCategory::AuthForbidden, "no access this global item");
pub const FORBIDEN: AtomicError = AtomicError(ErrorCategory::AuthForbidden, "access forbidden");
pub const INCOHERENT_WITH_STATE: AtomicError =
    AtomicError(ErrorCategory::BadRequest, "incoherent with state");
pub const NO_AUTH: AtomicError = AtomicError(ErrorCategory::AuthNone, "No authentification token");
pub const NOT_FOUND_ENTITY: AtomicError = AtomicError(ErrorCategory::NotFound, "not found entity");
pub const TOKEN_BASE64: AtomicError =
    AtomicError(ErrorCategory::BadRequest, "cannot parse base64 in token");
pub const TOKEN_OBSOLETE: AtomicError = AtomicError(ErrorCategory::BadRequest, "obsolete token");
pub const TOKEN_PREFIX: AtomicError =
    AtomicError(ErrorCategory::BadRequest, "unknown token prefix");
pub const TOKEN_SIGNATURE: AtomicError =
    AtomicError(ErrorCategory::BadRequest, "wrong token signature");
pub const TOKEN_WRONG_LENGTH: AtomicError =
    AtomicError(ErrorCategory::BadRequest, "token wrong lenght");
pub const WRONG_LOGIN: AtomicError =
    AtomicError(ErrorCategory::BadRequest, "wrong login information");
pub const WRONG_PATH: AtomicError = AtomicError(ErrorCategory::BadRequest, "wrong path format");

impl AtomicError {
    pub fn push(self, context: impl Into<ErrorStep>) -> Error {
        Error(vec![ErrorStep::Static(self.1), context.into()])
    }

    pub fn push_result(self, context: impl Into<ErrorStep>) -> Result<()> {
        Err(Error(vec![ErrorStep::Static(self.1), context.into()]))
    }
}

impl From<AtomicError> for Error {
    fn from(value: AtomicError) -> Self {
        Error(vec![ErrorStep::Static(value.1)])
    }
}

impl From<String> for ErrorStep {
    fn from(value: String) -> Self {
        Self::X(value)
    }
}

impl From<&'static str> for ErrorStep {
    fn from(value: &'static str) -> Self {
        Self::Static(value)
    }
}
