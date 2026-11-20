use std::{collections::BTreeMap, fmt::Write};

use crate::{
    chuncks_len,
    proto_http::{HTTPRequest, Method, StatusHTTP},
    Chunk, Chunks, DB,
};
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
    None,
}

pub async fn write_response<W, const N: usize, B>(
    mut w: W,
    response: Response<N>,
    db: &B,
    request: &HTTPRequest,
) -> std::io::Result<()>
where
    W: AsyncWrite + Unpin,
    B: DB,
{
    match &response.body {
        ResponseBody::None => {
            let mut buff = write_headers(&response);
            buff.push_str("Content-Length: 0\r\n");
            buff.push_str("\r\n");
            w.write_all(buff.as_bytes()).await?;
        }
        ResponseBody::Bytes(bytes) => {
            let mut buff = write_headers(&response);
            buff.push_str(format!("Content-Length: {}\r\n", bytes.len()).as_str());
            buff.push_str("\r\n");
            if request.method == Method::HEAD {
                return Ok(());
            }
            w.write_all(buff.as_bytes()).await?;
            w.write_all(&bytes).await?;
        }
        ResponseBody::Chunks { owner, chunks } => {
            if let Some(range) = get_range(&request.headers) {
                let total_len = chuncks_len(chunks);
                let range_start = range.0;
                let range_end = range.1.unwrap_or(total_len - 1);
                if range_start >= range_end || range_start >= total_len || range_end >= total_len {
                    let mut buff = write_headers(&Response {
                        status: StatusHTTP::RangeNotSatisfiable,
                        mime: response.mime,
                        header: response.header,
                        body: ResponseBody::None,
                    });
                    buff.push_str("Accept-Ranges: bytes\r\n");
                    w.write_all(buff.as_bytes()).await?;
                    return Ok(());
                }
                let mut buff = write_headers(&Response {
                    status: StatusHTTP::PartialContent,
                    mime: response.mime,
                    header: response.header,
                    body: ResponseBody::None,
                });
                buff.push_str("Accept-Ranges: bytes\r\n");
                let _ = write!(
                    buff,
                    "Content-Range: bytes {}-{}/{}\r\n",
                    range_start,
                    range_end,
                    chuncks_len(chunks)
                );
                let _ = write!(
                    buff,
                    "Content-Length: {}\r\n\r\n",
                    range_end + 1 - range_start,
                );
                w.write_all(buff.as_bytes()).await?;
                let mut chunks = Chunks::new(db, *owner, &chunks);
                chunks.set_range(range_start, range_end + 1);
                while let Some(data) = chunks.next().await? {
                    w.write_all(&data).await?;
                }
            } else {
                let mut buff = write_headers(&response);
                buff.push_str("Accept-Ranges: bytes\r\n");
                let _ = write!(buff, "Content-Length: {}\r\n\r\n", chuncks_len(chunks));
                w.write_all(buff.as_bytes()).await?;
                if request.method == Method::HEAD {
                    return Ok(());
                }
                let mut chunks = Chunks::new(db, *owner, &chunks);
                while let Some(data) = chunks.next().await? {
                    w.write_all(&data).await?;
                }
            }
        }
    }

    Ok(())
}

fn write_headers<const N: usize>(r: &Response<N>) -> String {
    let mut buff = String::new();
    buff.push_str("HTTP/1.1 ");
    buff.push_str(r.status.as_str());
    buff.push_str("\r\n");

    if !r.mime.is_empty() {
        buff.push_str("Content-Type: ");
        buff.push_str(r.mime);
        buff.push_str("\r\n");
    }

    for h in r.header.iter() {
        if let Some((key, value)) = h {
            buff.push_str(key);
            buff.push_str(": ");
            buff.push_str(value.as_str());
            buff.push_str("\r\n");
        }
    }

    buff
}

fn get_range(headers: &BTreeMap<String, String>) -> Option<(usize, Option<usize>)> {
    for h in headers {
        if h.0.eq_ignore_ascii_case("range") {
            return parse_range(&h.1);
        }
    }
    None
}

fn parse_range(range: &str) -> Option<(usize, Option<usize>)> {
    let (begin, end) = range.trim().strip_prefix("bytes=")?.split_once('-')?;

    let begin = if !begin.is_empty() {
        begin.parse::<usize>().ok()?
    } else {
        0
    };

    let end = if !end.is_empty() {
        Some(end.parse::<usize>().ok()?)
    } else {
        None
    };

    Some((begin, end))
}

#[test]
fn test_parse_range() {
    assert_eq!(parse_range("bytes=0-499"), Some((0, Some(499))));
    assert_eq!(parse_range("bytes=500-999"), Some((500, Some(999))));
    assert_eq!(parse_range("bytes=500-"), Some((500, None)));
    assert_eq!(parse_range("bytes=-500"), Some((0, Some(500))));
    assert_eq!(parse_range("pages="), None);
    assert_eq!(parse_range("bytes="), None);
    assert_eq!(parse_range(""), None);
}
