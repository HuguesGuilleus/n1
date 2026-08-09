use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use bytes::Bytes;
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::Config;
use crate::{AtomicError, Result, errs};

#[derive(Debug)]
pub struct ConfigMemoryMutex(Mutex<ConfigMemory>);

#[derive(Debug, Clone)]
struct ConfigMemory {
    fs: HashMap<(u32, u32), Bytes>,
    pages: HashMap<String, (&'static str, Bytes)>,
    id_increment: u32,
    now: Option<u64>,
}

impl ConfigMemoryMutex {
    pub fn new() -> Self {
        Self(Mutex::new(ConfigMemory {
            fs: HashMap::new(),
            pages: HashMap::new(),
            id_increment: ConfigMemoryMutex::RESERVED,
            now: Some(1780998183),
        }))
    }
}

#[async_trait]
impl Config for ConfigMemoryMutex {
    async fn fs_get(&self, eid: u32, oid: u32) -> Result<Bytes> {
        let mutex = self.0.lock().map_err(AtomicError::from)?;
        mutex
            .fs
            .get(&(eid, oid))
            .cloned()
            .ok_or(errs::NOT_FOUND.into())
    }

    async fn fs_set(&self, eid: u32, oid: u32, data: Bytes) -> Result<()> {
        let mut mutex = self.0.lock().map_err(AtomicError::from)?;
        mutex.fs.insert((eid, oid), data);
        Ok(())
    }

    async fn fs_rm_object(&self, eid: u32, oid: u32) -> Result<()> {
        let mut mutex = self.0.lock().map_err(AtomicError::from)?;
        mutex.fs.remove(&(eid, oid));
        Ok(())
    }

    async fn fs_rm_entity(&self, eid: u32) -> Result<()> {
        let mut mutex = self.0.lock().map_err(AtomicError::from)?;
        let mut oids = Vec::with_capacity(mutex.fs.len());
        for (&(local_eid, oid), _) in &mutex.fs {
            if eid == local_eid {
                oids.push(oid);
            }
        }
        for oid in oids {
            mutex.fs.remove(&(eid, oid));
        }
        Ok(())
    }

    async fn fs_new_object(&self, _: u32) -> Result<u32> {
        let mut mutex = self.0.lock().map_err(AtomicError::from)?;
        mutex.id_increment += 1;
        Ok(mutex.id_increment)
    }
    async fn fs_new_entity(&self) -> Result<u32> {
        let mut mutex = self.0.lock().map_err(AtomicError::from)?;
        mutex.id_increment += 1;
        Ok(mutex.id_increment)
    }

    async fn fs_scan(&self) -> Result<Vec<(u32, u32)>> {
        let mutex = self.0.lock().map_err(AtomicError::from)?;
        Ok(mutex.fs.keys().copied().collect::<Vec<(u32, u32)>>())
    }

    async fn fs_len(&self, eid: u32, oid: u32) -> Result<usize> {
        let mutex = self.0.lock().map_err(AtomicError::from)?;
        mutex
            .fs
            .get(&(eid, oid))
            .ok_or(errs::NOT_FOUND.into())
            .map(|bytes| bytes.len())
    }

    async fn obj_store<T: Serialize + Debug + Send>(
        &self,
        eid: u32,
        oid: u32,
        value: T,
    ) -> Result<()> {
        let s = serde_json::to_string(&value).map_err(|_| errs::DB_ENCODE)?;
        self.fs_set(eid, oid, Bytes::from_owner(s)).await?;
        Ok(())
    }
    async fn obj_fetch<T: DeserializeOwned + Default + Debug>(
        &self,
        eid: u32,
        oid: u32,
    ) -> Result<T> {
        let file = match self.fs_get(eid, oid).await {
            Ok(file) => file,
            Err(err) if err.atomic == errs::NOT_FOUND => return Ok(T::default()),
            Err(err) => return Err(err),
        };
        serde_json::from_reader(&file[..]).map_err(|_| errs::DB_DECODE.into())
    }

    async fn page_add(&self, path: &str, mime: &'static str, data: Bytes) -> Result<()> {
        let mut mutex = self.0.lock().map_err(AtomicError::from)?;
        mutex.pages.insert(path.to_string(), (mime, data));
        Ok(())
    }

    async fn page_get(&self, path: &str) -> Result<(&'static str, Bytes)> {
        let mutex = self.0.lock().map_err(AtomicError::from)?;
        match mutex.pages.get(path) {
            Some(page) => Ok(page.clone()),
            None => Err(errs::NOT_FOUND.into()),
        }
    }

    fn now(&self) -> Result<u64> {
        let mutex = self.0.lock().map_err(AtomicError::from)?;
        if let Some(now) = mutex.now {
            return Ok(now);
        }
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => Ok(duration.as_secs()),
            Err(_) => Err(errs::TIME_FAIL.into()),
        }
    }
}
