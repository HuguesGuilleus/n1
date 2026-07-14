pub use n1_tool::{Error, ErrorKind, Result};

macro_rules! E {
    ($n:ident,$k:ident, $m:expr) => {
        pub const $n: Error = Error {
            kind: ErrorKind::$k,
            msg: $m,
        };
    };
}

E!(NO_AUTH, BadRequest, "No authentification token");
E!(FORBIDEN_GLOBAL, BadRequest, "no access this global item");
E!(FORBIDEN_GROUP, BadRequest, "no access this group");
E!(FORBIDEN_OUTSIDE, BadRequest, "not in this group");
E!(TOKEN_PREFIX, BadRequest, "unknown token prefix");
E!(TOKEN_LEVEL, BadRequest, "unknown this token level");
E!(TOKEN_BASE64, BadRequest, "cannot parse base64 in token");
E!(TOKEN_WRONG_LENGTH, BadRequest, "token wrong lenght");
E!(TOKEN_OBSOLETE, BadRequest, "obsolete token");
E!(TOKEN_SIGNATURE, BadRequest, "wrong token signature");
E!(NOT_FOUND, NotFound, "not found");
