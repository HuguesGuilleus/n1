mod chunks;
mod db_mem;
mod db_onefile;

use std::fmt::Debug;

use bytes::Bytes;
use serde::{Serialize, de::DeserializeOwned};

pub use crate::errs::Result;
use crate::errs::{self, NOT_FOUND};
pub use chunks::{Chunk, Chunks, chuncks_len};
pub use db_mem::DBMemory;
pub use db_onefile::DBOneFile;

/// A zone reserved for standard app.
/// All objet smaller identifier are reserved for standard app.
const DB_RESERVED: u32 = 1024;

pub trait DB: Debug + Send + Sync {
    // FS operations
    fn fs_get(&self, eid: u32, oid: u32) -> impl Future<Output = Result<Bytes>> + Send;
    fn fs_set(&self, eid: u32, oid: u32, data: Bytes) -> impl Future<Output = Result<()>> + Send;
    fn fs_rm_object(&self, eid: u32, oid: u32) -> impl Future<Output = Result<()>> + Send;
    fn fs_rm_entity(&self, eid: u32) -> impl Future<Output = Result<()>> + Send;
    fn fs_new_entity(&self) -> impl Future<Output = Result<u32>> + Send;
    fn fs_new_object(&self, eid: u32) -> impl Future<Output = Result<u32>> + Send;

    // Meta informations
    fn fs_keys(&self) -> impl Future<Output = Result<Vec<(u32, u32)>>> + Send;

    // // Store/Fetch encoded object
    fn obj_set<T: Serialize + Debug + Send>(
        &self,
        eid: u32,
        oid: u32,
        value: T,
    ) -> impl Future<Output = Result<()>> + Send {
        let data = rmp_serde::to_vec(&value)
            .map_err(|err| errs::DB_ENCODE.push(err.to_string()))
            .unwrap();
        self.fs_set(eid, oid, Bytes::from(data))
    }

    fn obj_get<T: DeserializeOwned + Default + Debug>(
        &self,
        eid: u32,
        oid: u32,
    ) -> impl Future<Output = Result<T>> + Send {
        async move {
            match self.fs_get(eid, oid).await {
                Ok(data) => rmp_serde::from_slice(&data)
                    .map_err(|err| errs::DB_DECODE.push(err.to_string())),
                Err(err) if err.atomic == NOT_FOUND => Ok(T::default()),
                Err(err) => Err(err),
            }
        }
    }

    // Generated pages
    fn page_set(
        &self,
        path: &str,
        mime: &'static str,
        data: Bytes,
    ) -> impl Future<Output = Result<()>> + Send;
    fn page_get(&self, path: &str) -> impl Future<Output = Result<(&'static str, Bytes)>> + Send;

    // Get time, seconds since epoch.
    fn now(&self) -> Result<u64>;
}
