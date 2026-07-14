mod status;

use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::{Chunks, Config};
pub use status::StatusHTTP;

pub async fn response_bytes<W: AsyncWrite + Unpin>(
    mut w: W,
    status: StatusHTTP,
    mime: &str,
    data: &[u8],
) -> std::io::Result<()> {
    let mut buff = String::new();
    buff.push_str("HTTP/1.1 ");
    buff.push_str(status.as_str());
    buff.push_str("\r\n");
    buff.push_str("Content-Type: ");
    buff.push_str(mime);
    buff.push_str("\r\n");
    buff.push_str(format!("Content-Length: {}\r\n", data.len()).as_str());
    buff.push_str("\r\n");

    w.write_all(buff.as_bytes()).await?;
    w.write_all(&data).await?;

    Ok(())
}

pub async fn response_chunks<C: Config + Unpin, W: AsyncWrite + Unpin>(
    mut w: W,
    status: StatusHTTP,
    mime: &str,
    mut chunks: Chunks<C>,
) -> std::io::Result<()> {
    let mut buff = String::new();
    buff.push_str("HTTP/1.1 ");
    buff.push_str(status.as_str());
    buff.push_str("\r\n");
    buff.push_str("Content-Type: ");
    buff.push_str(mime);
    buff.push_str("\r\n");
    buff.push_str(format!("Content-Length: {}\r\n", chunks.len()).as_str());
    buff.push_str("\r\n");
    w.write_all(buff.as_bytes()).await?;

    while let Some(bytes) = chunks.next().await? {
        w.write_all(&bytes).await?;
    }

    Ok(())
}
