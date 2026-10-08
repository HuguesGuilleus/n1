use std::{
    collections::BTreeMap,
    future::ready,
    sync::atomic::{AtomicU32, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::errs::{self, Result};
use bytes::Bytes;
use tokio::sync::Mutex;

use super::DB;

#[derive(Debug)]
pub struct DBMemory {
    pub fs: Mutex<BTreeMap<(u32, u32), Bytes>>,
    pub inc: AtomicU32,
    pub page: Mutex<BTreeMap<String, (&'static str, Bytes)>>,
    pub now: Option<u64>,
}

impl DBMemory {
    pub fn new() -> Self {
        Self {
            fs: Mutex::new(BTreeMap::new()),
            inc: AtomicU32::new(super::DB_RESERVED),
            page: Mutex::new(BTreeMap::new()),
            now: Some(1780998183),
        }
    }
}

impl DB for DBMemory {
    fn fs_get(&self, eid: u32, oid: u32) -> impl Future<Output = super::Result<Bytes>> + Send {
        async move {
            let guard = self.fs.lock().await;
            guard
                .get(&(eid, oid))
                .cloned()
                .ok_or(errs::NOT_FOUND.into())
        }
    }

    fn fs_set(&self, eid: u32, oid: u32, data: Bytes) -> impl Future<Output = Result<()>> + Send {
        async move {
            let mut guard = self.fs.lock().await;
            guard.insert((eid, oid), data);
            Ok(())
        }
    }

    fn fs_rm_object(&self, eid: u32, oid: u32) -> impl Future<Output = Result<()>> + Send {
        async move {
            let mut guard = self.fs.lock().await;
            guard.remove(&(eid, oid));
            Ok(())
        }
    }

    fn fs_rm_entity(&self, eid: u32) -> impl Future<Output = Result<()>> + Send {
        async move {
            let mut guard = self.fs.lock().await;
            guard.retain(|&(e, _), _| e != eid);
            Ok(())
        }
    }

    fn fs_new_object(&self, _: u32) -> impl Future<Output = Result<u32>> + Send {
        ready(Ok(self.inc.fetch_add(1, Ordering::SeqCst)))
    }

    fn fs_new_entity(&self) -> impl Future<Output = Result<u32>> + Send {
        ready(Ok(self.inc.fetch_add(1, Ordering::SeqCst)))
    }

    fn fs_keys(&self) -> impl Future<Output = Result<Vec<(u32, u32)>>> + Send {
        async move {
            let guard = self.fs.lock().await;
            let result = guard.keys().cloned().collect();
            Ok(result)
        }
    }

    fn page_get(&self, path: &str) -> impl Future<Output = Result<(&'static str, Bytes)>> + Send {
        async move {
            let guard = self.page.lock().await;
            guard.get(path).cloned().ok_or(errs::NOT_FOUND.into())
        }
    }

    fn page_set(
        &self,
        path: &str,
        mime: &'static str,
        data: Bytes,
    ) -> impl Future<Output = Result<()>> + Send {
        async move {
            let mut guard = self.page.lock().await;
            guard.insert(path.to_string(), (mime, data));
            Ok(())
        }
    }

    fn now(&self) -> Result<u64> {
        if let Some(now) = self.now {
            return Ok(now);
        };
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => Ok(duration.as_secs()),
            Err(_) => Err(errs::TIME_FAIL.into()),
        }
    }
}
