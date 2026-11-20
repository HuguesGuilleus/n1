mod request;
mod response;
mod status;

use std::io;

pub use request::next_req;
pub use request::HTTPParser;
pub use request::HTTPRequest;
pub use request::Method;

pub use response::*;
pub use status::*;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::spawn;

use crate::mime;
use crate::mime::REDIRECT;
use crate::DBError;
use crate::DB;

pub trait HttpHandler<const N: usize, B: DB>: Clone + Send + Sync + 'static {
    type Error;

    fn handle<R: AsyncRead + Unpin + Send + Sync>(
        &self,
        request: &HTTPRequest,
        body: R,
    ) -> impl Future<Output = Result<Response<N>, Self::Error>> + Send + Sync;

    fn print_error(request: &HTTPRequest, err: Self::Error) -> Response<N>;

    fn db(&self) -> &B;
}

pub async fn run<const N: usize, B: DB, H: HttpHandler<N, B>>(
    listener: TcpListener,
    h: H,
) -> io::Result<()> {
    loop {
        let h = h.clone();
        let (socket, _) = listener.accept().await?;
        spawn(async move |h: H, mut socket: TcpStream| -> io::Result<()> {
            let (mut r, mut w) = socket.split();
            let mut parser = HTTPParser::new();
            while let Some(request_result) = next_req(&mut r, &mut parser).await {
                let (request, buffer, rest) = request_result
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))?;
                let body = AsyncReadExt::chain(buffer, (&mut r).take(rest));
                let response = h
                    .handle(&request, body)
                    .await
                    .unwrap_or_else(|err| H::print_error(&request, err));
                write_response(&mut w, response, h.db(), &request).await?;
            }
            Ok(())
        }(h, socket));
    }
}

pub async fn server_page(db: &impl DB, path: &str) -> Result<Response<1>, DBError> {
    let (m, bytes) = db.page_get(path).await?;
    if m == REDIRECT {
        let location = String::from_utf8(bytes.to_vec()).map_err(|_| DBError::DBInternal)?;
        return Ok(Response {
            status: StatusHTTP::Found,
            mime: mime::TEXT,
            header: [Some(("Location", location))],
            body: ResponseBody::Bytes(bytes),
        });
    }
    Ok(Response {
        status: StatusHTTP::OK,
        mime: m,
        header: [None],
        body: ResponseBody::Bytes(bytes),
    })
}
