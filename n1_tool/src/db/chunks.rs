use std::io;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::db::DBError;

use super::DB;

/** A big file with multiple chunks as content. */
#[derive(Debug)]
pub struct Chunks<'b, B: DB> {
    db: &'b B,
    owner: u32,
    chunks: &'b [Chunk],
    position: usize,
    end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Chunk {
    pub len: usize,
    pub oid: u32,
}

impl<'b, B: DB> Chunks<'b, B> {
    pub fn new(db: &'b B, owner: u32, chunks: &'b [Chunk]) -> Self {
        Self {
            db,
            owner,
            chunks,
            position: 0,
            end: chuncks_len(chunks),
        }
    }

    /// Move internal cursor at position arg to skip the begin.
    pub fn set_range(&mut self, begin: usize, end: usize) {
        self.position = begin;
        self.end = end;
    }

    pub async fn next(&mut self) -> io::Result<Option<Bytes>> {
        if self.position >= self.end {
            return Ok(None);
        }

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
            .db
            .fs_get(self.owner, chunck.oid)
            .await
            .map_err(|err| match err {
                DBError::NotFoundID { .. } | DBError::NotFoundPath { .. } => {
                    io::Error::from(io::ErrorKind::NotFound)
                }
                DBError::Io(kind) => io::Error::from(kind),
                _ => io::Error::from(io::ErrorKind::Other),
            })?;

        let mut data = data.slice(data.len() + self.position - cursor..);

        if self.end < cursor {
            data = data.slice(0..data.len() + self.end - cursor);
        }

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

    let chunks = vec![
        Chunk { len: 3, oid: 1 },
        Chunk { len: 3, oid: 2 },
        Chunk { len: 3, oid: 3 },
        Chunk { len: 3, oid: 4 },
    ];
    let mut c = Chunks::new(&db, 1, &chunks);

    assert_eq!(chuncks_len(&c.chunks), 12);
    c.set_range(5, 8);

    assert_eq!(c.next().await.unwrap(), Some(Bytes::from_static(b"c")));
    assert_eq!(c.next().await.unwrap(), Some(Bytes::from_static(b"de")));
    assert_eq!(c.next().await.unwrap(), None);
    assert_eq!(c.next().await.unwrap(), None);
}

pub async fn read_to_chunks<const L: usize>(
    db: &impl DB,
    owner: u32,
    mut reader: impl AsyncRead + Unpin,
) -> Result<Vec<Chunk>, DBError> {
    let mut buff = [0u8; L];
    let mut w = 0;
    let mut chunks = Vec::new();
    loop {
        let n = reader
            .read(&mut buff[w..])
            .await
            .map_err(|_| DBError::ClientConnexion)?;
        w += n;
        if w == L || (n == 0 && w != 0) {
            let oid = db.fs_new_object(owner).await?;
            db.fs_set(owner, oid, Bytes::copy_from_slice(&buff[..w]))
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
