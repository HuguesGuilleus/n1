use std::{io, sync::Arc};

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt};

use super::DB;
use crate::{Result, errs};

/** A big file with multiple chunks as content. */
#[derive(Debug)]
pub struct Chunks<B: DB> {
    config: Arc<B>,
    eid: u32,
    chunks: Vec<Chunk>,
    position: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct Chunk {
    pub len: usize,
    pub oid: u32,
}

impl<B: DB> Chunks<B> {
    pub fn new(config: Arc<B>, eid: u32, chunks: Vec<Chunk>) -> Self {
        Self {
            config,
            eid,
            chunks: chunks.to_vec(),
            position: 0,
        }
    }

    // Get total size of this chuncks.
    pub fn len(&self) -> usize {
        self.chunks.iter().map(|item| item.len).sum()
    }

    /// Move internal cursor at position arg to kip the begin.
    pub fn skip(&mut self, position: usize) {
        self.position = position;
    }

    pub async fn next(&mut self) -> io::Result<Option<Bytes>> {
        let mut cursor = 0;
        let chunck = self
            .chunks
            .iter()
            .skip_while(|chunck| {
                cursor += chunck.len;
                cursor <= self.position
            })
            .next();

        let chunck = match chunck {
            Some(chunck) => chunck,
            None => return Ok(None),
        };

        let data = self
            .config
            .fs_get(self.eid, chunck.oid)
            .await
            .map_err(|err| match err.atomic {
                errs::NOT_FOUND => io::Error::from(io::ErrorKind::NotFound),
                _ => io::Error::from(io::ErrorKind::Other),
            })?;

        let data = data.slice(data.len() + self.position - cursor..);
        self.position = cursor;

        Ok(Some(data))
    }
}

pub fn chuncks_len(chunks: &[Chunk]) -> usize {
    chunks.iter().map(|c| c.len).sum()
}

#[tokio::test]
async fn chunck() {
    let db = super::DBMemory::new();
    db.fs_set(1, 2, Bytes::from_static(b"abc")).await.unwrap();
    db.fs_set(1, 3, Bytes::from_static(b"def")).await.unwrap();

    let mut c = Chunks::new(
        Arc::new(db),
        1,
        vec![
            Chunk { len: 3, oid: 1 },
            Chunk { len: 3, oid: 2 },
            Chunk { len: 3, oid: 3 },
        ],
    );

    assert_eq!(c.len(), 9);
    assert_eq!(chuncks_len(&c.chunks), 9);
    c.skip(5);

    assert_eq!(c.next().await.unwrap(), Some(Bytes::from_static(b"c")));
    assert_eq!(c.next().await.unwrap(), Some(Bytes::from_static(b"def")));
    assert_eq!(c.next().await.unwrap(), None);
    assert_eq!(c.next().await.unwrap(), None);
}

pub async fn read_to_chunks<const L: usize>(
    db: &impl DB,
    eid: u32,
    mut reader: impl AsyncRead + Unpin,
) -> Result<Vec<Chunk>> {
    let mut buff = [0u8; L];
    let mut w = 0;
    let mut chunks = Vec::new();
    loop {
        let n = reader
            .read(&mut buff[w..])
            .await
            .map_err(|e| errs::IO_ERROR.push(e.to_string()))?;
        w += n;
        if w == L || (n == 0 && w != 0) {
            let oid = db.fs_new_object(eid).await?;
            db.fs_set(eid, oid, Bytes::copy_from_slice(&buff[..w]))
                .await?;
            chunks.push(Chunk { len: w, oid });
            w = 0;
        }
        if n == 0 {
            return Ok(chunks);
        }
    }
}
#[tokio::test]
async fn test_read_to_chunks() {
    let db = super::DBMemory::new();
    assert_eq!(
        vec![
            Chunk { len: 3, oid: 1024 },
            Chunk { len: 3, oid: 1025 },
            Chunk { len: 2, oid: 1026 }
        ],
        read_to_chunks::<3>(&db, 36, &b"12345678"[..])
            .await
            .unwrap()
    );

    let db = super::DBMemory::new();
    assert_eq!(
        vec![
            Chunk { len: 3, oid: 1024 },
            Chunk { len: 3, oid: 1025 },
            Chunk { len: 3, oid: 1026 }
        ],
        read_to_chunks::<3>(&db, 36, &b"123456789"[..])
            .await
            .unwrap()
    );
}
