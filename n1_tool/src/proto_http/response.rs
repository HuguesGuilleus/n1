use crate::{Chunks, Config, mime, proto_http::StatusHTTP};
use bytes::Bytes;
use tokio::io::{AsyncWrite, AsyncWriteExt};

pub struct Response<C: Config> {
    pub status: StatusHTTP,
    pub mime: &'static str,
    pub header: Option<(&'static str, String)>,
    pub body: ResponseBody<C>,
}
pub enum ResponseBody<C: Config> {
    Chunks(Chunks<C>),
    Bytes(Bytes),
}

pub async fn write_response(
    mut w: impl AsyncWrite + Unpin,
    r: Response<impl Config>,
) -> std::io::Result<()> {
    let mut buff = String::new();
    buff.push_str("HTTP/1.1 ");
    buff.push_str(r.status.as_str());
    buff.push_str("\r\n");

    if !r.mime.is_empty() {
        buff.push_str("Content-Type: ");
        buff.push_str(r.mime);
        buff.push_str("\r\n");
    }

    if let Some((key, value)) = r.header {
        buff.push_str(key);
        buff.push_str(": ");
        buff.push_str(value.as_str());
        buff.push_str("\r\n");
    }

    match r.body {
        ResponseBody::Bytes(bytes) => {
            buff.push_str(format!("Content-Length: {}\r\n", bytes.len()).as_str());
            buff.push_str("\r\n");
            w.write_all(buff.as_bytes()).await?;
            w.write_all(&bytes).await?;
        }
        ResponseBody::Chunks(mut chunks) => {
            buff.push_str(format!("Content-Length: {}\r\n", chunks.len()).as_str());
            buff.push_str("\r\n");
            w.write_all(buff.as_bytes()).await?;
            while let Some(data) = chunks.next().await? {
                w.write_all(&data).await?;
            }
        }
    }

    Ok(())
}

impl<C: Config> From<()> for Response<C> {
    fn from(_: ()) -> Self {
        Response {
            status: StatusHTTP::OK,
            mime: mime::HTML,
            header: None,
            body: ResponseBody::Bytes(Bytes::new()),
        }
    }
}
/// Consider the string as geenrated HTML.
impl<C: Config> From<String> for Response<C> {
    fn from(body: String) -> Self {
        Response {
            status: StatusHTTP::OK,
            mime: mime::HTML,
            header: None,
            body: ResponseBody::Bytes(Bytes::from_owner(body)),
        }
    }
}

impl<C: Config> From<Chunks<C>> for Response<C> {
    fn from(chunks: Chunks<C>) -> Self {
        Response {
            status: StatusHTTP::OK,
            mime: "",
            header: None,
            body: ResponseBody::Chunks(chunks),
        }
    }
}
