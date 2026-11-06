use std::collections::BTreeMap;

use bytes::Bytes;
use tokio::sync::Mutex;

use crate::{db::DBError, mime::REDIRECT};

#[derive(Debug)]
pub struct Page(Mutex<BTreeMap<String, (&'static str, Bytes)>>);

impl Page {
    pub fn new() -> Self {
        Self(Mutex::new(BTreeMap::new()))
    }

    pub async fn page_get(&self, path: &str) -> Result<(&'static str, Bytes), DBError> {
        let map = self.0.lock().await;
        map.get(path).cloned().ok_or(DBError::NotFoundPath {
            path: path.to_string(),
        })
    }

    pub async fn page_set(
        &self,
        path: &str,
        mime: &'static str,
        data: Bytes,
    ) -> Result<(), DBError> {
        let path_toggle = if path.ends_with("/") {
            path[..path.len() - 1].to_string()
        } else {
            let mut p = path.to_string();
            p.push('/');
            p
        };

        let mut map = self.0.lock().await;
        if data.len() == 0 {
            map.remove(path);
            map.remove(&path_toggle);
        } else {
            map.insert(path.to_string(), (mime, data));
            map.insert(path_toggle, (REDIRECT, Bytes::from_owner(path.to_string())));
        }

        Ok(())
    }

    pub async fn page_del(&self, prefix: &str) -> Result<(), DBError> {
        let mut map = self.0.lock().await;
        map.retain(|key, _| !key.starts_with(prefix));
        Ok(())
    }
}
