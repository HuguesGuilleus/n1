use std::collections::BTreeMap;
use std::sync::Arc;

use bytes::Bytes;
use n1_tool::proto_http::{
    server_page, HTTPRequest, HttpHandler, Response, ResponseBody, StatusHTTP,
};
use n1_tool::{mime, Chunk, DB};
use serde::Serialize;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::op::login::logout_console;
use crate::op::{self, print_error_html, Common, Token, DTO, URLDTO};
use crate::token::{token_decode, token_encode};
use crate::{errs, front, Error, ErrorCategory, ErrorStep, OpRequest, Result};

pub struct HTTPServerArc<B: DB>(pub Arc<HTTPServer<B>>);

impl<B: DB> Clone for HTTPServerArc<B> {
    fn clone(&self) -> Self {
        HTTPServerArc(self.0.clone())
    }
}

pub struct HTTPServer<B: DB> {
    pub db: B,
    pub common: Arc<Common>,
    pub token_key: [u8; 64],
}

impl<B> HttpHandler<1, B> for HTTPServerArc<B>
where
    B: DB + 'static,
{
    type Error = Error;

    fn handle<R: AsyncRead + Unpin + Send + Sync>(
        &self,
        r: &HTTPRequest,
        body: R,
    ) -> impl Future<Output = std::prelude::v1::Result<Response<1>, Self::Error>> + Send + Sync
    {
        let s = &self.0;
        async move {
            match r.path[1..].split_once('/').unwrap_or((&r.path[1..], "")).0 {
                "_style.css" => asset(mime::CSS, front::CSS),
                "_favicon.webp" => asset(mime::WEBP, front::FAVICON),
                "robots.txt" => asset(mime::TEXT, front::ROBOTSTXT),

                // Render HTML
                "_" => with_url(s, r, op::menu::menu_console).await,
                "_auth_global_admin" => with_url(s, r, op::auth::auth_global_admin).await,
                "_auth_login" => with_url(s, r, op::auth::login::login_console).await,
                "_bio" => with_url(s, r, op::bio::console).await,
                "_fs" => with_url(s, r, op::fs::fs_direct_index).await,
                "_home" => with_url(s, r, op::home::home_console).await,
                "_wiki_new" => with_url(s, r, op::wiki::render_priv_new).await,
                "_wiki_article" => with_url(s, r, op::wiki::render_priv_page).await,
                "_wiki" => with_url(s, r, op::wiki::render_priv_index).await,

                // Edit cookie
                "_auth_logout" => Ok(Response {
                    status: StatusHTTP::OK,
                    mime: mime::HTML,
                    header: [Some(("Set-Cookie", "auth=; Max-Age=0; Path=/".to_string()))],
                    body: ResponseBody::Bytes(Bytes::from_owner(logout_console(
                        self.0.common.lang,
                    ))),
                }),
                ":auth.login" => make_token(s, r, body, op::auth::login::login_action).await,

                // JSON actions API
                ":auth.admin.edit.name" => with_json(s, r, body, op::auth_admin_edit_name).await,
                ":auth.admin.new" => with_json(s, r, body, op::auth_admin_new).await,
                ":auth.check.access.read" => {
                    with_json(s, r, body, op::auth_check_access_read).await
                }
                ":auth.check.access.write" => {
                    with_json(s, r, body, op::auth_check_access_write).await
                }
                ":auth.check.auth" => with_json(s, r, body, op::auth_check_auth).await,
                ":auth.check.isadmin" => with_json(s, r, body, op::auth_check_isadmin).await,
                ":auth.get.id" => with_json(s, r, body, op::auth_get_id).await,
                ":bio.get" => with_json(s, r, body, op::bio::get).await,
                ":bio.set" => with_json(s, r, body, op::bio::set).await,
                ":fs.direct.mkdir" => with_json(s, r, body, op::fs::fs_direct_mkdir).await,
                ":fs.direct.mkfile" => with_json(s, r, body, op::fs::fs_direct_mkfile).await,
                ":fs.direct.move" => with_json(s, r, body, op::fs::fs_direct_move).await,
                ":fs.direct.resolve" => with_json(s, r, body, op::fs::fs_direct_resolve).await,
                ":fs.direct.rm" => with_json(s, r, body, op::fs::fs_direct_rm).await,
                ":fs.direct.tree" => with_json(s, r, body, op::fs::fs_direct_tree).await,
                ":home.get" => with_json(s, r, body, op::home::json_get).await,
                ":home.set" => with_json(s, r, body, op::home::json_set).await,
                ":wiki.article.get" => with_json(s, r, body, op::wiki::article_get).await,
                ":wiki.article.new" => with_json(s, r, body, op::wiki::article_new).await,
                ":wiki.article.rm" => with_json(s, r, body, op::wiki::article_rm).await,
                ":wiki.article.setContent" => {
                    with_json(s, r, body, op::wiki::article_set_content).await
                }
                ":wiki.article.setSlug" => with_json(s, r, body, op::wiki::article_set_slug).await,
                ":wiki.article.setTitle" => {
                    with_json(s, r, body, op::wiki::article_set_title).await
                }

                // Actions with request reader body or return body
                ";fs.direct.set" => {
                    with_reader(s, r, body, async |db, r, c| {
                        op::fs::fs_direct_set(db, r, c).await
                    })
                    .await
                }
                ";fs.direct.get" => {
                    with_reader(s, r, body, async |db, r, c| {
                        op::fs::fs_direct_get(db, r, c).await
                    })
                    .await
                }

                _ => server_page(&s.db, r.path.as_str())
                    .await
                    .map_err(|_| ErrorStep::NotFoundHandler.into()),
            }
        }
    }

    fn print_error(request: &HTTPRequest, err: Self::Error) -> Response<1> {
        let status = match err.category() {
            ErrorCategory::BadRequest => StatusHTTP::BadRequest,
            ErrorCategory::AuthForbidden => StatusHTTP::Forbidden,
            ErrorCategory::Internal => StatusHTTP::InternalServerError,
            ErrorCategory::AuthNone => StatusHTTP::Unauthorized,
            ErrorCategory::NotFound => StatusHTTP::NotFound,
        };

        let (mime, body) = if accept_html(&request.headers) {
            (mime::HTML, print_error_html(err, status))
        } else {
            let mut body = status.as_str().to_string();
            body.push_str("\n");
            err.0.iter().rev().for_each(|step| {
                body.push_str("--- ");
                body.push_str(&step.to_string());
                body.push_str("\n");
            });
            (mime::TEXT, body)
        };
        Response {
            status,
            mime,
            header: [None],
            body: ResponseBody::Bytes(Bytes::from_owner(body)),
        }
    }

    fn db(&self) -> &B {
        &self.0.db
    }
}

fn asset(mime: &'static str, bytes: &'static [u8]) -> Result<Response<1>> {
    Ok(Response {
        status: StatusHTTP::OK,
        mime: mime,
        header: [None],
        body: ResponseBody::Bytes(Bytes::from_static(bytes)),
    })
}

async fn with_url<C: DB, F: AsyncFn(&C, OpRequest<D>) -> Result<String>, D: URLDTO>(
    s: &HTTPServer<C>,
    r: &HTTPRequest,
    f: F,
) -> Result<Response<1>> {
    let token: Token = get_token(s, &r);

    let dto: D = URLDTO::url_decode(
        r.path[1..]
            .split_once('/')
            .map(|(_, data)| data)
            .unwrap_or(""),
    )?;
    dto.check()?;

    let html = f(
        &s.db,
        OpRequest {
            common: s.common.clone(),
            token,
            dto,
        },
    )
    .await?;

    Ok(Response {
        status: StatusHTTP::OK,
        mime: mime::HTML,
        header: [None],
        body: ResponseBody::Bytes(Bytes::from_owner(html)),
    })
}

async fn with_json<
    B: DB,
    R: AsyncRead + Unpin + Send + Sync,
    F: AsyncFn(&B, OpRequest<D>) -> Result<O>,
    O: Serialize,
    D: DTO,
>(
    s: &HTTPServer<B>,
    r: &HTTPRequest,
    mut body: R,
    f: F,
) -> Result<Response<1>> {
    let token = get_token(s, &r);

    let mut buf = Vec::new();
    body.read_to_end(&mut buf)
        .await
        .map_err(|err| ErrorStep::BodyRead(err))?;
    let data: &[u8] = match &buf[..] {
        b"" => b"null",
        _ => &buf,
    };
    let dto: D = serde_json::from_slice(data).map_err(|err| ErrorStep::BodyDecode(err))?;
    dto.check()?;

    let out = f(
        &s.db,
        OpRequest {
            common: s.common.clone(),
            token,
            dto,
        },
    )
    .await?;

    Ok(Response {
        status: StatusHTTP::OK,
        mime: mime::JSON,
        header: [None],
        body: ResponseBody::Bytes(Bytes::from_owner(serde_json::to_vec(&out).unwrap())),
    })
}

async fn with_reader<
    B: DB,
    F: AsyncFn(&B, OpRequest<D>, R) -> Result<(u32, Vec<Chunk>)>,
    D: URLDTO,
    R: AsyncRead + Unpin + Send,
>(
    s: &HTTPServer<B>,
    r: &HTTPRequest,
    body: R,
    f: F,
) -> Result<Response<1>> {
    let token = get_token(s, &r);

    let dto: D = URLDTO::url_decode(
        r.path[1..]
            .split_once('/')
            .map(|(_, data)| data)
            .unwrap_or(""),
    )?;
    dto.check()?;

    let (owner, chunks) = f(
        &s.db,
        OpRequest {
            common: s.common.clone(),
            token,
            dto,
        },
        body,
    )
    .await?;

    Ok(Response {
        status: StatusHTTP::OK,
        mime: mime::OCTET_STREAM,
        header: [None],
        body: ResponseBody::Chunks { owner, chunks },
    })
}

fn get_token(s: &HTTPServer<impl DB>, request: &HTTPRequest) -> Token {
    let cookie = request
        .headers
        .get("Cookie")
        .map(|c| c.as_str())
        .unwrap_or_default()
        .split("; ")
        .flat_map(|cookie| cookie.strip_prefix("auth="))
        .next();

    if let Some(cookie) = cookie {
        let now = match s.db.now() {
            Ok(time) => time,
            Err(_) => return Token::default(),
        };
        token_decode(&s.token_key[..], now, cookie)
            .inspect_err(|err| eprintln!("Cookie auth token fail: {:?}", err))
            .unwrap_or_default()
    } else {
        Token::default()
    }
}

async fn make_token<
    C: DB,
    R: AsyncRead + Unpin,
    F: AsyncFn(&C, OpRequest<D>) -> Result<Token>,
    D: DTO,
>(
    s: &HTTPServer<C>,
    r: &HTTPRequest,
    mut body: R,
    f: F,
) -> Result<Response<1>> {
    let mut buf = Vec::new();
    body.read_to_end(&mut buf)
        .await
        .map_err(|err| ErrorStep::BodyRead(err))?;
    let data: &[u8] = match &buf[..] {
        b"" => b"null",
        _ => &buf,
    };
    let dto = serde_json::from_slice(data).map_err(|_| errs::DECODE_REQUEST)?;

    let token: Token = f(
        &s.db,
        OpRequest {
            common: s.common.clone(),
            token: get_token(s, &r),
            dto,
        },
    )
    .await?;

    let mut header_value = token_encode(String::from("auth="), &token, &s.token_key, s.db.now()?);
    header_value.push_str("; Max-Age=604800; SameSite=Strict; Secure; HttpOnly");

    Ok(Response {
        status: StatusHTTP::OK,
        mime: "",
        header: [Some(("Set-Cookie", header_value))],
        body: ResponseBody::Bytes(Bytes::new()),
    })
}

fn accept_html(headers: &BTreeMap<String, String>) -> bool {
    let accept = headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("accept"))
        .map(|(_, value)| value.as_str())
        .unwrap_or_default();

    let format = accept
        .split(",")
        .map(|s| s.trim())
        .filter(|&s| {
            s == "text/html"
                || s.strip_prefix("text/html;").is_some()
                || s == "text/plain"
                || s.strip_prefix("text/plain;").is_some()
        })
        .next()
        .unwrap_or_default();

    format.strip_prefix("text/html").is_some()
}
#[test]
fn test_accept_html() {
    let mut headers = BTreeMap::new();
    headers.insert(
        "Accept".to_string(),
        "text/plain, text/html;q=0.9".to_string(),
    );
    assert!(!accept_html(&headers));

    let mut headers = BTreeMap::new();
    headers.insert(
        "Accept".to_string(),
        "text/css, text/html;q=0.9, text/plain;q=0.8".to_string(),
    );
    assert!(accept_html(&headers));

    headers.insert("Accept".to_string(), "application/json".to_string());
    assert!(!accept_html(&headers));
}
