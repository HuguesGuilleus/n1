pub use n1_tool::errs::*;

macro_rules! E {
    ($n:ident,$k:ident, $m:expr) => {
        pub const $n: Error = Error {
            kind: ErrorKind::$k,
            msg: $m,
        };
    };
}

E!(DECODE_REQUEST, BadRequest, "cannot decode request data");
E!(FIELD_CONTENT, BadRequest, "the field 'content' is empty");
E!(FIELD_DESC, BadRequest, "the field 'desc' is empty");
E!(FIELD_NAME, BadRequest, "the field 'name' is empty");
E!(FIELD_PASSWORD, BadRequest, "the field 'password' is empty");
E!(FIELD_TITLE, BadRequest, "the field 'title' is empty");
E!(FORBIDEN_GLOBAL, BadRequest, "no access this global item");
E!(FORBIDEN_GROUP, BadRequest, "no access this group");
E!(FORBIDEN_OUTSIDE, BadRequest, "not in this group");
E!(NO_AUTH, BadRequest, "No authentification token");
E!(TOKEN_BASE64, BadRequest, "cannot parse base64 in token");
E!(TOKEN_LEVEL, BadRequest, "unknown this token level");
E!(TOKEN_OBSOLETE, BadRequest, "obsolete token");
E!(TOKEN_PREFIX, BadRequest, "unknown token prefix");
E!(TOKEN_SIGNATURE, BadRequest, "wrong token signature");
E!(TOKEN_WRONG_LENGTH, BadRequest, "token wrong lenght");
E!(WRONG_LOGIN, BadRequest, "wrong login information");
