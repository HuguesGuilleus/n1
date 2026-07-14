mod chunks;
mod config_mem;

use std::fmt::Debug;

use async_trait::async_trait;
use bytes::Bytes;
use serde::{Serialize, de::DeserializeOwned};

pub use crate::errs::Result;
pub use chunks::{Chunk, Chunks};
pub use config_mem::ConfigMemoryMutex;

#[async_trait]
pub trait Config: Debug + Send + Sync {
    /// A zone reserved for standard app.
    /// All objet smaller identifier are reserved for standard app.
    const RESERVED: u32 = 1024;

    // FS operations
    async fn fs_get(&self, eid: u32, oid: u32) -> Result<Bytes>;
    async fn fs_set(&self, eid: u32, oid: u32, data: Bytes) -> Result<()>;
    async fn fs_rm_object(&self, eid: u32, oid: u32) -> Result<()>;
    async fn fs_rm_entity(&self, eid: u32) -> Result<()>;
    async fn fs_new_entity(&self) -> Result<u32>;
    async fn fs_new_object(&self, eid: u32) -> Result<u32>;

    // Meta informations:
    async fn fs_len(&self, eid: u32, oid: u32) -> Result<usize>;
    async fn fs_scan(&self) -> Result<Vec<(u32, u32)>>;

    // Store/Fetch encoded object
    async fn obj_store<T: Serialize + Debug + Send>(
        &self,
        eid: u32,
        oid: u32,
        value: T,
    ) -> Result<()>;
    async fn obj_fetch<T: DeserializeOwned + Default + Debug>(
        &self,
        eid: u32,
        oid: u32,
    ) -> Result<T>;

    // Generated pages
    async fn page_add(&self, path: &str, mime: &'static str, data: Bytes) -> Result<()>;
    async fn page_get(&self, path: &str) -> Result<(&'static str, Bytes)>;

    // Get time, seconds since epoch.
    fn now(&self) -> Result<u64>;
}
