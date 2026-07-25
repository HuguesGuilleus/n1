use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use tokio::net::TcpListener;
use tokio::spawn;

use crate::op::{self, DTO, OpRequest, OpResponse, OpServer, Token};
use crate::token::{token_decode, token_encode};
use crate::{Result, errs, front};
use n1_tool::proto_http::{
    HTTPParser, HTTPRequest, StatusHTTP, response_bytes, response_chunks, response_cookie,
    response_empty,
};
use n1_tool::{Config, Error, ErrorKind, mime};

pub struct HTTPServer<C: Config> {
    pub op: OpServer<C>,
    pub key: [u8; 64],
}

pub async fn run<C: Config + Unpin + 'static>(server: Arc<HTTPServer<C>>) -> io::Result<()> {
    let server = server.clone();
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
                handle_wrap(&server, request, &mut w).await?;
            }
            Ok(())
        }());
    }
}

pub async fn handle_wrap(
    server: &HTTPServer<impl Config + Unpin>,
    request: HTTPRequest<impl AsyncRead + Unpin>,
    w: impl AsyncWrite + Unpin,
) -> io::Result<()> {
    match handle_op(server, request).await {
        Ok(OpResponse::Ok) => response_empty(w, StatusHTTP::OK).await,
        Ok(OpResponse::Bytes(mime, data)) => response_bytes(w, StatusHTTP::OK, mime, &data).await,
        Ok(OpResponse::Chunks(chunks)) => {
            response_chunks(w, StatusHTTP::OK, mime::TEXT, chunks).await
        }
        Ok(OpResponse::Token(token)) => {
            response_cookie(
                w,
                "auth",
                token_encode(
                    &token,
                    &server.key,
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                ),
            )
            .await
        }
        Err(Error {
            kind: ErrorKind::SubIO | ErrorKind::Internal,
            msg,
        }) => {
            response_bytes(
                w,
                StatusHTTP::InternalServerError,
                mime::TEXT,
                &Bytes::from_static(msg.as_bytes()),
            )
            .await
        }
        Err(Error {
            kind: ErrorKind::BadRequest,
            msg,
        }) => response_bytes(w, StatusHTTP::BadRequest, mime::TEXT, msg.as_bytes()).await,
        Err(Error {
            kind: ErrorKind::NotFound,
            msg,
        }) => response_bytes(w, StatusHTTP::NotFound, mime::TEXT, msg.as_bytes()).await,
    }
}

pub async fn handle_op<R: AsyncRead + Unpin, C: Config>(
    serv: &HTTPServer<C>,
    r: HTTPRequest<R>,
) -> Result<OpResponse<C>> {
    match r.path.as_str() {
        "/_style.css" => Ok(OpResponse::Bytes(mime::CSS, Bytes::from_static(front::CSS))),
        "/favicon.webp" => Ok(OpResponse::Bytes(
            mime::WEBP,
            Bytes::from_static(front::FAVICON),
        )),
        "/robots.txt" => Ok(OpResponse::Bytes(
            mime::TEXT,
            Bytes::from_static(front::ROBOTSTXT),
        )),

        "/io" => op::big(&serv.op, from_url(serv, r)?).await,

        "/_home/" => op::home::page_console(&serv.op, &from_url(serv, r)?).await,
        "/:home/" => op::home::json_edit(&serv.op, &from_body(serv, r).await?).await,

        "/:login" => op::user::login::login(&serv.op, &from_body(serv, r).await?).await,

        p => {
            let (mime, data) = serv.op.config.page_get(p).await?;
            Ok(OpResponse::Bytes(mime, data))
        }
    }
}

pub fn from_url<R: AsyncRead, C: Config, D: DTO>(
    server: &HTTPServer<C>,
    request: HTTPRequest<R>,
) -> Result<OpRequest<D>> {
    let token = get_token(&server.key, &request);

    let data = match request.path[1..].split_once('/') {
        Some((_, "")) | None => "null",
        Some((_, data)) => data,
    };
    let dto = serde_json::from_str(data).map_err(|_| errs::DECODE_REQUEST)?;

    Ok(OpRequest { token, dto })
}

pub async fn from_body<R: AsyncRead + Unpin, C: Config, D: DTO>(
    server: &HTTPServer<C>,
    mut request: HTTPRequest<R>,
) -> Result<OpRequest<D>> {
    let token = get_token(&server.key, &request);

    let mut buf = Vec::new();
    request.body.read_to_end(&mut buf).await?;
    let data: &[u8] = match &buf[..] {
        b"" => b"null",
        _ => &buf,
    };
    let dto = serde_json::from_slice(data).map_err(|_| errs::DECODE_REQUEST)?;

    Ok(OpRequest { token, dto })
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
