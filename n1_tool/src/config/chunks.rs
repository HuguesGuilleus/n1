use std::{io, sync::Arc};

use bytes::Bytes;

use super::Config;
use crate::errs;

/** A big file with multiple chunks as content. */
pub struct Chunks<C: Config> {
    config: Arc<C>,
    eid: u32,
    chunks: Vec<Chunk>,
    position: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chunk {
    pub len: usize,
    pub oid: u32,
}

impl<C: Config> Chunks<C> {
    pub fn new(config: Arc<C>, eid: u32, chunks: &[Chunk]) -> Self {
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
            .map_err(|err| match err {
                errs::NOT_FOUND => io::Error::from(io::ErrorKind::NotFound),
                _ => io::Error::from(io::ErrorKind::Other),
            })?;

        let data = data.slice(data.len() + self.position - cursor..);
        self.position = cursor;

        Ok(Some(data))
    }
}

#[tokio::test]
async fn chunck() {
    use super::ConfigMemoryMutex;

    let config = ConfigMemoryMutex::new();
    config
        .fs_set(1, 2, Bytes::from_static(b"abc"))
        .await
        .unwrap();
    config
        .fs_set(1, 3, Bytes::from_static(b"def"))
        .await
        .unwrap();

    let mut c = Chunks::new(
        Arc::new(config),
        1,
        &[
            Chunk { len: 3, oid: 1 },
            Chunk { len: 3, oid: 2 },
            Chunk { len: 3, oid: 3 },
        ],
    );

    assert_eq!(c.len(), 9);
    c.skip(5);

    assert_eq!(c.next().await.unwrap(), Some(Bytes::from_static(b"c")));
    assert_eq!(c.next().await.unwrap(), Some(Bytes::from_static(b"def")));
    assert_eq!(c.next().await.unwrap(), None);
    assert_eq!(c.next().await.unwrap(), None);
}
