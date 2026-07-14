use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicI64;

use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::spawn;

use crate::op::{self, OpRequest, OpResponse, OpServer};
use crate::{Result, errs};
use n1_tool::proto_http::{HTTPParser, HTTPRequest, response_bytes};
use n1_tool::{Error, ErrorKind, mime};

pub async fn run() -> io::Result<()> {
    let server = Arc::new(OpServer {
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
    server: &OpServer,
    request: HTTPRequest<impl AsyncRead + Unpin>,
    mut w: impl AsyncWrite + Unpin,
) -> io::Result<()> {
    match handle_op(server, request).await {
        Ok(OpResponse::HTML(html)) => response_bytes(w, mime::HTML, html.as_bytes()).await,
        Ok(OpResponse::Chunks(mut chunks)) => {
            let mut buff = String::new();
            buff.push_str("HTTP/1.1 200 OK\r\n");
            buff.push_str("Content-Type: text/plain; charset=utf-8\r\n");
            buff.push_str(format!("Content-Length: {}\r\n", chunks.len()).as_str());
            buff.push_str("\r\n");
            w.write_all(buff.as_bytes()).await?;

            while let Some(data) = chunks.next() {
                w.write_all(&data).await?;
            }

            Ok(())
        }
        Err(Error {
            kind: ErrorKind::NotFound,
            ..
        }) => response_bytes(w, mime::TEXT, "Not Found".as_bytes()).await,
        Err(err, ..) => response_bytes(w, mime::TEXT, err.msg.as_bytes()).await,
    }
}

pub async fn handle_op<R: AsyncRead + Unpin>(
    server: &OpServer,
    request: HTTPRequest<R>,
) -> Result<OpResponse> {
    match request.path.as_str() {
        "/io" => Ok(op::big(server, parse_request(server, request).await?).await),
        "/" => op::add(server, parse_request(server, request).await?).await,
        _ => Err(errs::NOT_FOUND),
    }
}

pub async fn parse_request<R>(_server: &OpServer, _request: HTTPRequest<R>) -> Result<OpRequest> {
    Ok(OpRequest { nb: 2 })
}
