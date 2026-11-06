use std::{
    io::ErrorKind,
    time::{SystemTime, UNIX_EPOCH},
};

use bytes::Bytes;
use tokio::{
    fs::{File, create_dir_all, read, remove_dir_all, remove_file, try_exists, write},
    io::AsyncReadExt,
    sync::Mutex,
};

use crate::{
    DB, Page,
    db::{DB_RESERVED, DBError},
};

#[derive(Debug)]
pub struct DBDirectory {
    root: String,
    urandom: Mutex<File>,
    pages: Page,
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
            pages: Page::new(),
        }
    }

    async fn random(&self) -> Result<u32, DBError> {
        let mut urandom = self.urandom.lock().await;
        let mut buff = [0u8; 4];
        urandom
            .read_exact(&mut buff)
            .await
            .map_err(|err| DBError::Io(err.kind()))?;
        Ok(u32::from_ne_bytes(buff))
    }
}

impl DB for DBDirectory {
    fn fs_get(
        &self,
        owner: u32,
        object: u32,
    ) -> impl Future<Output = Result<Bytes, DBError>> + Send {
        async move {
            match read(format!("{}/{}/{}", self.root, owner, object)).await {
                Ok(data) => Ok(Bytes::from_owner(data)),
                Err(err) if err.kind() == ErrorKind::NotFound => {
                    return Err(DBError::NotFoundID { owner, object });
                }
                Err(err) => Err(DBError::Io(err.kind())),
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
            create_dir_all(format!("{}/{}", self.root, eid))
                .await
                .map_err(|err| DBError::Io(err.kind()))?;
            write(format!("{}/{}/{}", self.root, eid, oid), data)
                .await
                .map_err(|err| DBError::Io(err.kind()))?;
            Ok(())
        }
    }

    fn fs_rm_object(&self, eid: u32, oid: u32) -> impl Future<Output = Result<(), DBError>> + Send {
        async move {
            match remove_file(format!("{}/{}/{}", self.root, eid, oid)).await {
                Ok(_) => Ok(()),
                Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
                Err(err) => Err(DBError::Io(err.kind())),
            }
        }
    }

    fn fs_rm_owner(&self, eid: u32) -> impl Future<Output = Result<(), DBError>> + Send {
        async move {
            remove_dir_all(format!("{}/{}", self.root, eid))
                .await
                .map_err(|err| DBError::Io(err.kind()))
        }
    }

    fn fs_new_owner(&self) -> impl Future<Output = Result<u32, DBError>> + Send {
        async move {
            loop {
                let id = self.random().await?;
                let exist = try_exists(format!("{}/{}", self.root, id))
                    .await
                    .map_err(|err| DBError::Io(err.kind()))?;
                if !exist {
                    return Ok(id);
                }
            }
        }
    }

    fn fs_new_object(&self, eid: u32) -> impl Future<Output = Result<u32, DBError>> + Send {
        async move {
            loop {
                let id = self.random().await?;
                if id < DB_RESERVED {
                    continue;
                }
                let exist = try_exists(format!("{}/{}/{}", self.root, eid, id))
                    .await
                    .map_err(|err| DBError::Io(err.kind()))?;
                if !exist {
                    return Ok(id);
                }
            }
        }
    }

    fn fs_keys(&self) -> impl Future<Output = Result<Vec<(u32, u32)>, DBError>> + Send {
        async { Ok(vec![]) }
    }

    fn page_get(
        &self,
        path: &str,
    ) -> impl Future<Output = Result<(&'static str, Bytes), DBError>> + Send {
        self.pages.page_get(path)
    }

    fn page_set(
        &self,
        path: &str,
        mime: &'static str,
        data: Bytes,
    ) -> impl Future<Output = Result<(), DBError>> + Send {
        self.pages.page_set(path, mime, data)
    }

    fn page_del(&self, prefix: &str) -> impl Future<Output = Result<(), DBError>> + Send {
        self.pages.page_del(prefix)
    }

    fn now(&self) -> Result<u64, DBError> {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => Ok(duration.as_secs()),
            Err(_) => Err(DBError::GetTime),
        }
    }
}
