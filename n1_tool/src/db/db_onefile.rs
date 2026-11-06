use std::{
    collections::BTreeMap,
    io::{ErrorKind, SeekFrom},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
    vec,
};

use bytes::Bytes;
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    sync::Mutex,
};

use super::{DB, DB_RESERVED};
use crate::{Page, db::DBError};

const MAGIC: [u8; 8] = *b"Kventry_";

/// An experimental in development database with one OS file.
#[derive(Debug)]
pub struct DBOneFile {
    file: Mutex<DBFile>,
    page: Page,
}

#[derive(Debug)]
struct DBFile {
    write: File,
    read: File,
    offset: u64,
    entries: BTreeMap<(u32, u32), DBOneFileEntry>,
    inc: u32,
}

#[derive(Debug, Clone, Copy)]
struct DBOneFileEntry {
    offset: u64,
    size: u64,
}

impl DBOneFile {
    pub async fn open(path: impl AsRef<Path>) -> Result<DBOneFile, DBError> {
        // Open write file
        let mut write = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path)
            .await
            .map_err(|err| DBError::Io(err.kind()))?;

        if write
            .stream_position()
            .await
            .map_err(|err| DBError::Io(err.kind()))?
            == 0
        {
            write
                .write_all(&MAGIC)
                .await
                .map_err(|err| DBError::Io(err.kind()))?;
        }

        // Open read file
        let mut read = OpenOptions::new()
            .read(true)
            .open(&path)
            .await
            .map_err(|err| DBError::Io(err.kind()))?;
        let mut magic = [0u8; 8];
        read.read_exact(&mut magic)
            .await
            .map_err(|err| DBError::Io(err.kind()))?;
        if magic != MAGIC {
            return Err(DBError::MagicNumber {
                got: magic,
                expected: MAGIC,
            });
        }

        // Index all file
        let mut entries: BTreeMap<(u32, u32), DBOneFileEntry> = BTreeMap::new();
        let mut offset: u64 = 8;
        loop {
            let mut buff = [0u8; 16];
            if let Err(err) = read.read_exact(&mut buff[..]).await {
                if err.kind() == ErrorKind::UnexpectedEof {
                    break;
                } else {
                    return Err(DBError::Io(err.kind()));
                }
            }
            let meta = u128::from_be_bytes(buff);
            let size = (meta >> 64) as u64;
            let eid = (meta >> 32) as u32;
            let oid = meta as u32;
            if size > 0 {
                entries.insert(
                    (eid, oid),
                    DBOneFileEntry {
                        offset: 16 + offset,
                        size,
                    },
                );
            } else {
                entries.remove(&(eid, oid));
            }
            read.seek(SeekFrom::Current(size as i64))
                .await
                .map_err(|err| DBError::Io(err.kind()))?;
            offset += 16 + size;
        }

        Ok(DBOneFile {
            file: Mutex::new(DBFile {
                write,
                read,
                offset,
                entries,
                inc: DB_RESERVED,
            }),
            page: Page::new(),
        })
    }
}

impl DB for DBOneFile {
    fn fs_get(
        &self,
        owner: u32,
        object: u32,
    ) -> impl Future<Output = Result<Bytes, DBError>> + Send {
        async move {
            let mut guard = self.file.lock().await;
            if let Some(&entry) = guard.entries.get(&(owner, object)) {
                let mut buff = vec![0u8; entry.size as usize];
                guard
                    .read
                    .seek(SeekFrom::Start(entry.offset))
                    .await
                    .map_err(|_| DBError::DBInternal)?;
                guard
                    .read
                    .read_exact(&mut buff)
                    .await
                    .map_err(|_| DBError::DBInternal)?;
                Ok(Bytes::from_owner(buff))
            } else {
                Err(DBError::NotFoundID { owner, object })
            }
        }
    }

    fn fs_set(
        &self,
        eid: u32,
        oid: u32,
        data: Bytes,
    ) -> impl Future<Output = Result<(), DBError>> + Send {
        async move {
            let mut guard = self.file.lock().await;
            let mut chunck = vec![0u8; 16 + data.len()];
            let meta = (data.len() as u128) << 64 | (eid as u128) << 32 | oid as u128;
            chunck[..16].copy_from_slice(&meta.to_be_bytes());
            chunck[16..].copy_from_slice(&data);
            guard
                .write
                .write_all(&chunck)
                .await
                .map_err(|err| DBError::Io(err.kind()))?;
            let entry = DBOneFileEntry {
                offset: guard.offset + 16,
                size: data.len() as u64,
            };
            guard.entries.insert((eid, oid), entry);
            guard.offset += 16 + data.len() as u64;
            Ok(())
        }
    }

    fn fs_rm_object(&self, eid: u32, oid: u32) -> impl Future<Output = Result<(), DBError>> + Send {
        async move {
            let mut guard = self.file.lock().await;
            guard.entries.remove(&(eid, oid));
            let meta = (eid as u128) << 32 | oid as u128;
            guard
                .write
                .write_all(&meta.to_be_bytes())
                .await
                .map_err(|err| DBError::Io(err.kind()))
        }
    }
    fn fs_rm_owner(&self, eid: u32) -> impl Future<Output = Result<(), DBError>> + Send {
        async move {
            let mut guard = self.file.lock().await;
            let removed_entries: Vec<u8> = guard
                .entries
                .keys()
                .copied()
                .filter(|(local_eid, _)| *local_eid == eid)
                .flat_map(|(oid, eid)| {
                    (((eid as u128) << 32) | (oid as u128))
                        .to_be_bytes()
                        .into_iter()
                })
                .collect();

            guard
                .write
                .write_all(&removed_entries)
                .await
                .map_err(|err| DBError::Io(err.kind()))?;

            guard.entries.retain(|&(local_eid, _), _| local_eid != eid);
            Ok(())
        }
    }

    fn fs_new_owner(&self) -> impl Future<Output = Result<u32, DBError>> + Send {
        async move {
            let mut guard = self.file.lock().await;
            while guard
                .entries
                .keys()
                .any(|(local_eid, _)| *local_eid == guard.inc)
            {
                guard.inc += 1;
            }
            Ok(guard.inc)
        }
    }
    fn fs_new_object(&self, eid: u32) -> impl Future<Output = Result<u32, DBError>> + Send {
        async move {
            let mut guard = self.file.lock().await;
            while guard.entries.contains_key(&(eid, guard.inc)) {
                guard.inc += 1;
            }
            Ok(guard.inc)
        }
    }

    fn fs_keys(&self) -> impl Future<Output = Result<Vec<(u32, u32)>, DBError>> + Send {
        async move {
            let guard = self.file.lock().await;
            Ok(guard.entries.keys().copied().collect())
        }
    }

    fn page_get(
        &self,
        path: &str,
    ) -> impl Future<Output = Result<(&'static str, Bytes), DBError>> + Send {
        self.page.page_get(path)
    }

    fn page_set(
        &self,
        path: &str,
        mime: &'static str,
        data: Bytes,
    ) -> impl Future<Output = Result<(), DBError>> + Send {
        self.page.page_set(path, mime, data)
    }

    fn page_del(&self, prefix: &str) -> impl Future<Output = Result<(), DBError>> + Send {
        self.page.page_del(prefix)
    }

    fn now(&self) -> Result<u64, DBError> {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => Ok(duration.as_secs()),
            Err(_) => Err(DBError::GetTime),
        }
    }
}
