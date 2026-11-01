use std::{ collections::BTreeMap, io::ErrorKind::NotFound};

use bytes::Bytes;
use tokio::{
    fs::{File, create_dir_all, read, remove_dir_all, remove_file, try_exists, write},
    io::AsyncReadExt,
    sync::{Mutex, RwLock},
};

use crate::{DB, Error, Result, errs, mime::REDIRECT};

#[derive(Debug)]
pub struct DBDirectory {
    root: String,
    urandom: Mutex<File>,
    pages: RwLock<BTreeMap<String, (&'static str, Bytes)>>,
}

impl DBDirectory {
    pub async fn new(mut root: String) -> Self {
        if root.ends_with('/') {
            root.pop();
        }
        if root == "" {
            root = ".".to_string();
        }
        Self {
            root,
            urandom: Mutex::new(File::open("/dev/urandom").await.unwrap()),
            pages: RwLock::new(BTreeMap::new()),
        }
    }

    async fn random(&self) -> Result<u32> {
        let mut urandom = self.urandom.lock().await;
        let mut buff = [0u8; 4];
        urandom.read_exact(&mut buff).await?;
        Ok(u32::from_ne_bytes(buff))
    }
}

impl DB for DBDirectory {
    fn fs_get(&self, eid: u32, oid: u32) -> impl Future<Output = super::Result<Bytes>> + Send {
        async move {
            match read(format!("{}/{}/{}", self.root, eid, oid)).await {
                Ok(data) => Ok(Bytes::from_owner(data)),
                Err(err) if err.kind() == NotFound => return Err(errs::NOT_FOUND.into()),
                Err(err) => Err(err.into()),
            }
        }
    }

    fn fs_set(
        &self,
        eid: u32,
        oid: u32,
        data: Bytes,
    ) -> impl Future<Output = super::Result<()>> + Send {
        async move {
            create_dir_all(format!("{}/{}", self.root, eid))
                .await
                .map_err(Error::from)?;
            write(format!("{}/{}/{}", self.root, eid, oid), data)
                .await
                .map_err(Error::from)?;
            Ok(())
        }
    }

    fn fs_rm_object(&self, eid: u32, oid: u32) -> impl Future<Output = super::Result<()>> + Send {
        async move {
            match remove_file(format!("{}/{}/{}", self.root, eid, oid)).await {
                Ok(_) => Ok(()),
                Err(err) if err.kind() == NotFound => Ok(()),
                Err(err) => Err(err.into()),
            }
        }
    }

    fn fs_rm_entity(&self, eid: u32) -> impl Future<Output = super::Result<()>> + Send {
        async move {
            remove_dir_all(format!("{}/{}", self.root, eid))
                .await
                .map_err(Error::from)
        }
    }

    fn fs_new_entity(&self) -> impl Future<Output = super::Result<u32>> + Send {
        async move {
            loop {
                let id = self.random().await?;
                if try_exists(format!("{}/{}", self.root, id)).await? == false {
                    return Ok(id);
                }
            }
        }
    }

    fn fs_new_object(&self, eid: u32) -> impl Future<Output = super::Result<u32>> + Send {
        async move {
            loop {
                let id = self.random().await?;
                if try_exists(format!("{}/{}/{}", self.root, eid, id)).await? == false {
                    return Ok(id);
                }
            }
        }
    }

    fn fs_keys(&self) -> impl Future<Output = super::Result<Vec<(u32, u32)>>> + Send {
        async { Ok(vec![]) }
    }

    fn page_get(
        &self,
        path: &str,
    ) -> impl Future<Output = super::Result<(&'static str, Bytes)>> + Send {
        async move {
            let pages = self.pages.read().await;
            pages.get(path).cloned().ok_or(errs::NOT_FOUND.into())
        }
    }

    fn page_set(
        &self,
        path: &str,
        mime: &'static str,
        data: Bytes,
    ) -> impl Future<Output = super::Result<()>> + Send {
        async move {
            let mut pages = self.pages.write().await;
            let path_toggle = if path.ends_with("/") {
                path[..path.len() - 1].to_string()
            } else {
                let mut p = path.to_string();
                p.push('/');
                p
            };
            if data.len() == 0 {
                pages.remove(path);
                pages.remove(&path_toggle);
                return Ok(());
            }
            pages.insert(path.to_string(), (mime, data));
            pages.insert(path_toggle, (REDIRECT, Bytes::from_owner(path.to_string())));
            Ok(())
        }
    }

    fn now(&self) -> Result<u64> {
        // match SystemTime::now().duration_since(UNIX_EPOCH) {
        //     Ok(duration) => Ok(duration.as_secs()),
        //     Err(_) => Err(errs::TIME_FAIL.into()),
        // }
        Ok(1780998183)
    }
}
