mod chunks;
mod db_directory;
mod db_mem;
mod db_onefile;
mod page;

use std::fmt::Debug;

use bytes::Bytes;
use serde::{Serialize, de::DeserializeOwned};

pub use chunks::{Chunk, Chunks, chuncks_len, read_to_chunks};
pub use db_directory::DBDirectory;
pub use db_mem::DBMemory;
pub use db_onefile::DBOneFile;
pub use page::Page;

/// A zone reserved for standard app.
/// All objet smaller identifier are reserved for standard app.
const DB_RESERVED: u32 = 1024;

pub trait DB: Debug + Send + Sync {
    // FS operations
    fn fs_get(
        &self,
        owner: u32,
        object: u32,
    ) -> impl Future<Output = Result<Bytes, DBError>> + Send;
    fn fs_set(
        &self,
        owner: u32,
        object: u32,
        data: Bytes,
    ) -> impl Future<Output = Result<(), DBError>> + Send;
    fn fs_rm_object(
        &self,
        owner: u32,
        object: u32,
    ) -> impl Future<Output = Result<(), DBError>> + Send;
    fn fs_rm_owner(&self, owner: u32) -> impl Future<Output = Result<(), DBError>> + Send;
    fn fs_new_owner(&self) -> impl Future<Output = Result<u32, DBError>> + Send;
    fn fs_new_object(&self, owner: u32) -> impl Future<Output = Result<u32, DBError>> + Send;

    /// Get all keys of the database.
    fn fs_keys(&self) -> impl Future<Output = Result<Vec<(u32, u32)>, DBError>> + Send;

    /// Store/Fetch encoded object
    fn obj_set<T: Serialize + Debug + Send>(
        &self,
        owner: u32,
        object: u32,
        value: T,
    ) -> impl Future<Output = Result<(), DBError>> + Send {
        async move {
            let data = rmp_serde::to_vec_named(&value).map_err(|err| DBError::Encode(err))?;
            self.fs_set(owner, object, Bytes::from(data)).await
        }
    }

    /// Fetch data an decode it.
    /// If data not found, generate default data.
    fn obj_get<T: DeserializeOwned + Default + Debug>(
        &self,
        owner: u32,
        object: u32,
    ) -> impl Future<Output = Result<T, DBError>> + Send {
        async move {
            match self.fs_get(owner, object).await {
                Ok(data) => rmp_serde::from_slice(&data).map_err(|err| DBError::Decode(err)),
                Err(DBError::NotFoundID { .. } | DBError::NotFoundPath { .. }) => Ok(T::default()),
                Err(err) => Err(err),
            }
        }
    }

    /// Save a page
    fn page_set(
        &self,
        path: &str,
        mime: &'static str,
        data: Bytes,
    ) -> impl Future<Output = Result<(), DBError>> + Send;

    /// Get a page
    fn page_get(
        &self,
        path: &str,
    ) -> impl Future<Output = Result<(&'static str, Bytes), DBError>> + Send;
    /// Delete all page with the prefix.
    fn page_del(&self, prefix: &str) -> impl Future<Output = Result<(), DBError>> + Send;

    // Get time, seconds since epoch.
    fn now(&self) -> Result<u64, DBError>;
}

/// Error from the data base operation.
#[derive(Debug)]
pub enum DBError {
    ClientConnexion,
    Decode(rmp_serde::decode::Error),
    Encode(rmp_serde::encode::Error),
    GetTime,
    DBInternal,
    Io(std::io::ErrorKind),
    MagicNumber { got: [u8; 8], expected: [u8; 8] },
    NotFoundID { owner: u32, object: u32 },
    NotFoundPath { path: String },
}
