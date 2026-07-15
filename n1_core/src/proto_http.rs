use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::spawn;

use crate::op::{self, OpRequest, OpResponse, OpServer, init};
use crate::{Result, errs, front};
use n1_tool::proto_http::{
    HTTPParser, HTTPRequest, Method, StatusHTTP, response_bytes, response_chunks,
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
    server: &OpServer<C>,
    request: HTTPRequest<R>,
) -> Result<OpResponse<C>> {
    match (request.method, request.path.as_str()) {
        // Assets
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

        (_, "/io") => op::big(server, parse_request(server, request).await?).await,
        (_, "/add") => op::add(server, parse_request(server, request).await?).await,
        (Method::GET, p) => {
            let (mime, data) = server.config.page_get(p).await?;
            Ok(OpResponse::Bytes(mime, data))
        }
        _ => Err(errs::NOT_FOUND),
    }
}

pub async fn parse_request<R: AsyncRead, C: Config>(
    _server: &OpServer<C>,
    _request: HTTPRequest<R>,
) -> Result<OpRequest> {
    Ok(OpRequest { nb: 2 })
}
