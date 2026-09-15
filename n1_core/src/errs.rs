pub use n1_tool::errs::*;

macro_rules! E {
    ($n:ident,$k:ident, $m:expr) => {
        pub const $n: AtomicError = AtomicError {
            kind: ErrorKind::$k,
            message: $m,
        };
    };
}

E!(DECODE_REQUEST, BadRequest, "cannot decode request data");
E!(EXPECT_USER, BadRequest, "expected a user");
E!(FIELD_EMPTY, BadRequest, "empty field");
E!(FIELD_CONTENT, BadRequest, "empty 'content' field");
E!(FIELD_DESC, BadRequest, "empty 'desc' field");
E!(FIELD_ENTITY, BadRequest, "empty entity ID 'eid' field");
E!(FIELD_ID, BadRequest, "identifier field is zero");
E!(FIELD_NAME, BadRequest, "empty 'name' field");
E!(FIELD_NOT_EMPTY, BadRequest, "expected empty data");
E!(FIELD_OID, BadRequest, "empty object ID 'oid' field");
E!(FIELD_PASSWORD, BadRequest, "empty 'password' field");
E!(FIELD_STR, BadRequest, "empty 'str' field");
E!(FIELD_TITLE, BadRequest, "empty 'title' field");
E!(FORBIDEN_GLOBAL, Forbiden, "no access this global item");
E!(FORBIDEN_GROUP, Forbiden, "no access this group");
E!(NO_AUTH, BadRequest, "No authentification token");
E!(NOT_FOUND_ENTITY, NotFound, "not found entity");
E!(NOT_FOUND_USER, NotFound, "not found user");
E!(TOKEN_BASE64, BadRequest, "cannot parse base64 in token");
E!(TOKEN_OBSOLETE, BadRequest, "obsolete token");
E!(TOKEN_PREFIX, BadRequest, "unknown token prefix");
E!(TOKEN_SIGNATURE, BadRequest, "wrong token signature");
E!(TOKEN_WRONG_LENGTH, BadRequest, "token wrong lenght");
E!(WRONG_LOGIN, BadRequest, "wrong login information");
