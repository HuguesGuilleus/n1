use crate::{Chunks, DB, proto_http::StatusHTTP};
use bytes::Bytes;
use tokio::io::{AsyncWrite, AsyncWriteExt};

pub struct Response<C: DB> {
    pub status: StatusHTTP,
    pub mime: &'static str,
    pub header: Option<(&'static str, String)>,
    pub body: ResponseBody<C>,
}
pub enum ResponseBody<C: DB> {
    Chunks(Chunks<C>),
    Bytes(Bytes),
}

pub async fn write_response(
    mut w: impl AsyncWrite + Unpin,
    r: Response<impl DB>,
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
