use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use tokio::net::TcpListener;
use tokio::spawn;

use crate::op::{self, DTO, OpRequest, OpResponse, OpServer, Token};
use crate::{Result, errs, front};
use n1_tool::proto_http::{
    HTTPParser, HTTPRequest, Method, StatusHTTP, response_bytes, response_chunks, response_empty,
};
use n1_tool::{Config, Error, ErrorKind, mime};

pub async fn run<C: Config + Unpin + 'static>(server: Arc<OpServer<C>>) -> io::Result<()> {
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
    server: &OpServer<impl Config + Unpin>,
    request: HTTPRequest<impl AsyncRead + Unpin>,
    w: impl AsyncWrite + Unpin,
) -> io::Result<()> {
    match handle_op(server, request).await {
        Ok(OpResponse::Ok) => response_empty(w, StatusHTTP::OK).await,
        Ok(OpResponse::Bytes(mime, data)) => response_bytes(w, StatusHTTP::OK, mime, &data).await,
        Ok(OpResponse::Chunks(chunks)) => {
            response_chunks(w, StatusHTTP::OK, mime::TEXT, chunks).await
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
    serv: &OpServer<C>,
    r: HTTPRequest<R>,
) -> Result<OpResponse<C>> {
    match (r.method, r.path.as_str()) {
        // Asset
        (Method::GET, "/_style.css") => {
            Ok(OpResponse::Bytes(mime::CSS, Bytes::from_static(front::CSS)))
        }
        (Method::GET, "/favicon.webp") => Ok(OpResponse::Bytes(
            mime::WEBP,
            Bytes::from_static(front::FAVICON),
        )),
        (Method::GET, "/robots.txt") => Ok(OpResponse::Bytes(
            mime::TEXT,
            Bytes::from_static(front::ROBOTSTXT),
        )),

        // Console
        (Method::GET, "/_home/") => {
            op::home::console(serv, &parse_request_url(serv, r).await?).await
        }

        // Action
        (_, "/io") => op::big(serv, parse_request_url(serv, r).await?).await,
        (Method::PUT, "/_home/") => op::home::edit(serv, &parse_request_body(serv, r).await?).await,

        // Page
        (Method::GET, p) => {
            let (mime, data) = serv.config.page_get(p).await?;
            Ok(OpResponse::Bytes(mime, data))
        }
        _ => Err(errs::NOT_FOUND),
    }
}

pub async fn parse_request_url<R: AsyncRead, C: Config, D: DTO>(
    _server: &OpServer<C>,
    request: HTTPRequest<R>,
) -> Result<OpRequest<D>> {
    let data = match request.path[1..].split_once('/') {
        Some((_, "")) | None => "null",
        Some((_, data)) => data,
    };
    let dto = serde_json::from_str(data).map_err(|_| errs::DECODE_REQUEST)?;

    Ok(OpRequest {
        token: Token::test_alice(),
        dto,
    })
}

pub async fn parse_request_body<R: AsyncRead + Unpin, C: Config, D: DTO>(
    _server: &OpServer<C>,
    mut request: HTTPRequest<R>,
) -> Result<OpRequest<D>> {
    let mut buf = Vec::new();
    request.body.read_to_end(&mut buf).await?;

    let data: &[u8] = match &buf[..] {
        b"" => b"null",
        _ => &buf,
    };

    let dto = serde_json::from_slice(data).map_err(|_| errs::DECODE_REQUEST)?;

    Ok(OpRequest {
        token: Token::test_alice(),
        dto,
    })
}
