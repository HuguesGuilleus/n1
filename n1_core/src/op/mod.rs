mod home;
mod token;

use std::sync::{Arc, atomic::AtomicI64};

use bytes::Bytes;
use n1_tool::{Chunk, Chunks, Config, mime};

use crate::Result;
pub use token::*;

pub const OID_GLOBAL_HOME: u32 = 1;

pub struct OpServer<C> {
    pub config: Arc<C>,
    pub nb: AtomicI64,
}

pub struct OpRequest {
    pub nb: i64,
}

pub type OpResult<C> = Result<OpResponse<C>>;

pub enum OpResponse<C: Config> {
    Bytes(&'static str, Bytes),
    Chunks(Chunks<C>),
}

pub async fn init<C: Config + Unpin>(server: &OpServer<C>) -> Result<()> {
    home::init(server).await?;

    Ok(())
}

pub async fn add<C: Config>(server: &OpServer<C>, req: OpRequest) -> OpResult<C> {
    let nb = req.nb
        + server
            .nb
            .fetch_add(req.nb, std::sync::atomic::Ordering::AcqRel);

    Ok(OpResponse::Bytes(
        mime::HTML,
        Bytes::from_owner(format!("{}", nb)),
    ))
}

pub async fn big<C: Config>(server: &OpServer<C>, _req: OpRequest) -> OpResult<C> {
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
