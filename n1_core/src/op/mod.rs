mod token;

use std::sync::atomic::AtomicI64;

use bytes::Bytes;

use crate::Result;
pub use token::*;

pub struct OpServer {
    pub nb: AtomicI64,
}

pub struct OpRequest {
    pub nb: i64,
}

pub enum OpResponse {
    HTML(String),
    Chunks(Chunks),
}

pub async fn add(server: &OpServer, req: OpRequest) -> Result<OpResponse> {
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let nb = req.nb
        + server
            .nb
            .fetch_add(req.nb, std::sync::atomic::Ordering::AcqRel);

    Ok(OpResponse::HTML(format!("{}", nb)))
}

pub async fn big(_server: &OpServer, _req: OpRequest) -> OpResponse {
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    OpResponse::Chunks(Chunks::new())
}

pub struct Chunks {
    pub a: Option<Bytes>,
    pub b: Option<Bytes>,
}

impl Chunks {
    fn new() -> Self {
        Chunks {
            a: Some(Bytes::from_static(b"abc\n")),
            b: Some(Bytes::from_static(b"def\n")),
        }
    }

    pub fn len(&self) -> usize {
        self.a.as_ref().map(|a| a.len()).unwrap_or(0)
            + self.b.as_ref().map(|b| b.len()).unwrap_or(0)
    }

    pub fn next(&mut self) -> Option<Bytes> {
        if let Some(data) = self.a.clone() {
            self.a = None;
            Some(data)
        } else if let Some(data) = self.b.clone() {
            self.b = None;
            Some(data)
        } else {
            None
        }
    }
}
