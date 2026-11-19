use crate::{proto_http::StatusHTTP, Chunk, Chunks, DB};
use bytes::Bytes;
use tokio::io::{AsyncWrite, AsyncWriteExt};

pub struct Response<const N: usize> {
    pub status: StatusHTTP,
    pub mime: &'static str,
    pub header: [Option<(&'static str, String)>; N],
    pub body: ResponseBody,
}
pub enum ResponseBody {
    Chunks { owner: u32, chunks: Vec<Chunk> },
    Bytes(Bytes),
}

pub async fn write_response<W, const N: usize, B>(
    mut w: W,
    r: Response<N>,
    db: &B,
) -> std::io::Result<()>
where
    W: AsyncWrite + Unpin,
    B: DB,
{
    let mut buff = String::new();
    buff.push_str("HTTP/1.1 ");
    buff.push_str(r.status.as_str());
    buff.push_str("\r\n");

    if !r.mime.is_empty() {
        buff.push_str("Content-Type: ");
        buff.push_str(r.mime);
        buff.push_str("\r\n");
    }

    for h in r.header {
        if let Some((key, value)) = h {
            buff.push_str(key);
            buff.push_str(": ");
            buff.push_str(value.as_str());
            buff.push_str("\r\n");
        }
    }

    match r.body {
        ResponseBody::Bytes(bytes) => {
            buff.push_str(format!("Content-Length: {}\r\n", bytes.len()).as_str());
            buff.push_str("\r\n");
            w.write_all(buff.as_bytes()).await?;
            w.write_all(&bytes).await?;
        }
        ResponseBody::Chunks { owner, chunks } => {
            let mut chunks = Chunks::new(db, owner, chunks);
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
