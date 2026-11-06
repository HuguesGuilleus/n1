use std::{
    collections::BTreeMap,
    future::ready,
    sync::atomic::{AtomicU32, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{Page, db::DBError};
use bytes::Bytes;
use tokio::sync::Mutex;

use super::DB;

#[derive(Debug)]
pub struct DBMemory {
    pub fs: Mutex<BTreeMap<(u32, u32), Bytes>>,
    pub inc: AtomicU32,
    pub page: Page,
    pub now: Option<u64>,
}

impl DBMemory {
    pub fn new() -> Self {
        Self {
            fs: Mutex::new(BTreeMap::new()),
            inc: AtomicU32::new(super::DB_RESERVED),
            page: Page::new(),
            now: Some(1780998183),
        }
    }
}

impl DB for DBMemory {
    fn fs_get(
        &self,
        owner: u32,
        object: u32,
    ) -> impl Future<Output = Result<Bytes, DBError>> + Send {
        async move {
            let guard = self.fs.lock().await;
            guard
                .get(&(owner, object))
                .cloned()
                .ok_or(DBError::NotFoundID { owner, object })
        }
    }

    fn fs_set(
        &self,
        owner: u32,
        object: u32,
        data: Bytes,
    ) -> impl Future<Output = Result<(), DBError>> + Send {
        async move {
            let mut guard = self.fs.lock().await;
            guard.insert((owner, object), data);
            Ok(())
        }
    }

    fn fs_rm_object(
        &self,
        owner: u32,
        object: u32,
    ) -> impl Future<Output = Result<(), DBError>> + Send {
        async move {
            let mut guard = self.fs.lock().await;
            guard.remove(&(owner, object));
            Ok(())
        }
    }

    fn fs_rm_owner(&self, owner: u32) -> impl Future<Output = Result<(), DBError>> + Send {
        async move {
            let mut guard = self.fs.lock().await;
            guard.retain(|&(e, _), _| e != owner);
            Ok(())
        }
    }

    fn fs_new_object(&self, _: u32) -> impl Future<Output = Result<u32, DBError>> + Send {
        ready(Ok(self.inc.fetch_add(1, Ordering::SeqCst)))
    }

    fn fs_new_owner(&self) -> impl Future<Output = Result<u32, DBError>> + Send {
        ready(Ok(self.inc.fetch_add(1, Ordering::SeqCst)))
    }

    fn fs_keys(&self) -> impl Future<Output = Result<Vec<(u32, u32)>, DBError>> + Send {
        async move {
            let guard = self.fs.lock().await;
            let result = guard.keys().cloned().collect();
            Ok(result)
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
        if let Some(now) = self.now {
            return Ok(now);
        };
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => Ok(duration.as_secs()),
            Err(_) => Err(DBError::GetTime),
        }
    }
}
