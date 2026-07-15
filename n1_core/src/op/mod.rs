mod compo;
pub mod home;
mod token;

use std::fmt::Debug;
use std::sync::Arc;

use bytes::Bytes;
use n1_tool::{Chunk, Chunks, Config};
use serde::de::DeserializeOwned;

use crate::Result;
pub use token::*;

pub const OID_GLOBAL_HOME: u32 = 1;

pub struct OpServer<C> {
    pub config: Arc<C>,
}

pub struct OpRequest<D: DTO> {
    pub token: Token,
    pub dto: D,
}

pub trait DTO: DeserializeOwned + Debug {
    fn check(&self) -> Result<()>;
}

pub type OpResult<C> = Result<OpResponse<C>>;

pub enum OpResponse<C: Config> {
    Bytes(&'static str, Bytes),
    Chunks(Chunks<C>),
    Ok,
}

impl DTO for () {
    fn check(&self) -> Result<()> {
        Ok(())
    }
}

pub async fn init<C: Config + Unpin>(server: &OpServer<C>) -> Result<()> {
    home::init(server).await?;

    Ok(())
}

pub async fn big<C: Config>(server: &OpServer<C>, _req: OpRequest<()>) -> OpResult<C> {
    let b1 = Bytes::from_static(b"Hello ");
    let b2 = Bytes::from_static(b"World!\r\n");
    server.config.fs_set(42, 1, b1.clone()).await?;
    server.config.fs_set(42, 2, b2.clone()).await?;

    Ok(OpResponse::Chunks(Chunks::new(
        server.config.clone(),
        42,
        &[
            Chunk {
                len: b1.len(),
                oid: 1,
            },
            Chunk {
                len: b2.len(),
                oid: 2,
            },
        ],
    )))
}
