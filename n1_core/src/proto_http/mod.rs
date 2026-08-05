use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use n1_html::{H, Html};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::net::TcpListener;
use tokio::spawn;

use crate::op::{self, DTO, OpServer, Token};
use crate::token::{token_decode, token_encode};
use crate::{OpRequest, Result, errs, front};
use n1_tool::proto_http::{
    HTTPParser, HTTPRequest, Response, ResponseBody, StatusHTTP, write_response,
};
use n1_tool::{Config, Error, ErrorKind, mime};

pub struct HTTPServer<C: Config> {
    pub op: OpServer<C>,
    pub key: [u8; 64],
}

pub async fn run<C: Config + Unpin + 'static>(server: Arc<HTTPServer<C>>) -> io::Result<()> {
    let addr: SocketAddr = ([127, 0, 0, 1], 8000).into();
    let listener = TcpListener::bind(addr).await?;
    loop {
        let server = server.clone();
        let (mut stream, _) = listener.accept().await?;

        spawn(async move || -> io::Result<()> {
            let server = server;
            let (r, mut w) = stream.split();
            let mut parser = HTTPParser::new(r);
            while let Ok(request) = parser.next().await {
                let response = handle(&server, request).await.unwrap_or_else(print_error);
                write_response(&mut w, response).await?;
            }
            Ok(())
        }());
    }
}

pub async fn handle<R: AsyncRead + Unpin, C: Config>(
    s: &HTTPServer<C>,
    r: HTTPRequest<R>,
) -> Result<Response<C>> {
    match r.path[1..].split_once('/').unwrap_or((&r.path[1..], "")).0 {
        "_style.css" => asset(mime::CSS, front::CSS),
        "_favicon.webp" => asset(mime::WEBP, front::FAVICON),
        "robots.txt" => asset(mime::TEXT, front::ROBOTSTXT),

        ":io" => with_url(s, r, op::big).await,

        // Render HTML
        "_home" => with_url(s, r, op::home::page_console).await,
        "_dropbox" => with_url(s, r, op::dropbox::page).await,
        "_" => with_url(s, r, op::menu::page).await,

        // Actions with no return
        ":home" => with_body(s, r, op::home::json_edit).await,
        ":login" => make_token(s, r, op::user::login::login).await,
        ":dropbox.text.add" => with_body(s, r, op::dropbox::text_add).await,
        ":dropbox.text.rm" => with_body(s, r, op::dropbox::text_rm).await,

        // Serve generated files
        _ => {
            let (mime, bytes) = s.op.config.page_get(r.path.as_str()).await?;
            Ok(Response {
                status: StatusHTTP::OK,
                mime,
                header: None,
                body: ResponseBody::Bytes(bytes),
            })
        }
    }
}

fn asset<C: Config>(mime: &'static str, bytes: &'static [u8]) -> Result<Response<C>> {
    Ok(Response {
        status: StatusHTTP::OK,
        mime: mime,
        header: None,
        body: ResponseBody::Bytes(Bytes::from_static(bytes)),
    })
}

async fn with_url<
    C: Config,
    R: AsyncRead,
    F: AsyncFn(&OpServer<C>, OpRequest<D>) -> Result<O>,
    O: Into<Response<C>>,
    D: DTO,
>(
    s: &HTTPServer<C>,
    r: HTTPRequest<R>,
    f: F,
) -> Result<Response<C>> {
    let token = get_token(&s.key, &r);

    let data = match r.path[1..].split_once('/') {
        Some((_, "")) | None => "null",
        Some((_, data)) => data,
    };
    let dto = serde_json::from_str(data).map_err(|_| errs::DECODE_REQUEST)?;

    Ok(f(&s.op, OpRequest { token, dto }).await?.into())
}

async fn with_body<
    C: Config,
    R: AsyncRead + Unpin,
    F: AsyncFn(&OpServer<C>, OpRequest<D>) -> Result<O>,
    O: Into<Response<C>>,
    D: DTO,
>(
    s: &HTTPServer<C>,
    mut r: HTTPRequest<R>,
    f: F,
) -> Result<Response<C>> {
    let token = get_token(&s.key, &r);

    let mut buf = Vec::new();
    r.body.read_to_end(&mut buf).await?;
    let data: &[u8] = match &buf[..] {
        b"" => b"null",
        _ => &buf,
    };
    let dto = serde_json::from_slice(data).map_err(|_| errs::DECODE_REQUEST)?;

    Ok(f(&s.op, OpRequest { token, dto }).await?.into())
}

fn get_token<B: AsyncRead>(key: &[u8], request: &HTTPRequest<B>) -> Token {
    let cookie = request
        .headers
        .get("Cookie")
        .map(|c| c.as_str())
        .unwrap_or("")
        .split("; ")
        .filter(|cookie| cookie.starts_with("auth="))
        .next();

    if let Some(cookie) = cookie {
        token_decode(
            &cookie[5..],
            key,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
        .inspect_err(|err| eprintln!("Cookie auth token fail: {:?}", err))
        .ok()
        .unwrap_or_default()
    } else {
        Token::default()
    }
}

async fn make_token<
    C: Config,
    R: AsyncRead + Unpin,
    F: AsyncFn(&OpServer<C>, OpRequest<D>) -> Result<Token>,
    D: DTO,
>(
    s: &HTTPServer<C>,
    mut r: HTTPRequest<R>,
    f: F,
) -> Result<Response<C>> {
    let mut buf = Vec::new();
    r.body.read_to_end(&mut buf).await?;
    let data: &[u8] = match &buf[..] {
        b"" => b"null",
        _ => &buf,
    };
    let dto = serde_json::from_slice(data).map_err(|_| errs::DECODE_REQUEST)?;

    let token: Token = f(
        &s.op,
        OpRequest {
            token: Token::default(),
            dto,
        },
    )
    .await?;

    let mut header_value = String::from("auth=");
    token_encode(
        &mut header_value,
        &token,
        &s.key,
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
    );
    header_value.push_str("; SameSite=Strict; Secure; HttpOnly");

    Ok(Response {
        status: StatusHTTP::OK,
        mime: "",
        header: Some(("Set-Cookie", header_value)),
        body: ResponseBody::Bytes(Bytes::new()),
    })
}

fn print_error<C: Config>(err: Error) -> Response<C> {
    let status = match err.kind {
        ErrorKind::BadRequest => StatusHTTP::BadRequest,
        ErrorKind::Forbiden => StatusHTTP::Forbidden,
        ErrorKind::Internal => StatusHTTP::InternalServerError,
        ErrorKind::SubIO => StatusHTTP::InternalServerError,
        ErrorKind::NotFound => StatusHTTP::NotFound,
    };

    let body = ResponseBody::Bytes(Bytes::from_owner(
        [H - "html lang=en"
            + [H - "head" + front::HEAD]
            + [H - "body.m"
                + "\r\n"
                + [H - "h1" + status.as_str()]
                + [H - "div.mv" + err.msg]
                + "\r\n"
                + [H - "div.fh.gap"
                    + [H - "a.bl href=/ " + "///"]
                    + [H - "a.bl href=/_login" + "Login"]
                    + ""]
                + "\r\n"]
            + ""]
        .render_page(),
    ));

    Response {
        status,
        mime: mime::HTML,
        header: None,
        body,
    }
}
