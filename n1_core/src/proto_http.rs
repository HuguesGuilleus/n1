use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicI64;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::spawn;

use crate::op::{self, OpRequest, OpResponse, OpServer};
use crate::{Result, errs};
use n1_tool::proto_http::{HTTPParser, HTTPRequest, StatusHTTP, response_bytes, response_chunks};
use n1_tool::{Config, ConfigMemoryMutex, Error, ErrorKind, mime};

pub async fn run() -> io::Result<()> {
    let server = Arc::new(OpServer {
        config: Arc::new(ConfigMemoryMutex::new()),
        nb: AtomicI64::new(14),
    });

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
        Ok(OpResponse::HTML(html)) => {
            response_bytes(w, StatusHTTP::OK, mime::HTML, html.as_bytes()).await
        }
        Ok(OpResponse::Chunks(chunks)) => {
            response_chunks(w, StatusHTTP::OK, mime::TEXT, chunks).await
        }
        Err(Error {
            kind: ErrorKind::SubIO | ErrorKind::Internal,
            ..
        }) => {
            response_bytes(
                w,
                StatusHTTP::InternalServerError,
                mime::TEXT,
                b"Internal server error",
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
    match request.path.as_str() {
        "/io" => op::big(server, parse_request(server, request).await?).await,
        "/" => op::add(server, parse_request(server, request).await?).await,
        _ => Err(errs::NOT_FOUND),
    }
}

pub async fn parse_request<R: AsyncRead, C: Config>(
    _server: &OpServer<C>,
    _request: HTTPRequest<R>,
) -> Result<OpRequest> {
    Ok(OpRequest { nb: 2 })
}
