mod compo;
pub mod home;
pub mod menu;
mod token;
pub mod user;

use std::sync::{Arc, RwLock};
use std::{collections::BTreeMap, fmt::Debug};

use bytes::Bytes;
use n1_tool::{Chunk, Chunks, Config};
use serde::de::DeserializeOwned;

use crate::Result;
pub use token::*;
use user::User;

pub const OID_GLOBAL_USER: u32 = 1;
pub const OID_GLOBAL_HOME: u32 = 3;

pub struct OpServer<C> {
    pub config: Arc<C>,
    pub user: RwLock<BTreeMap<u32, User>>,
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
    Token(Token),
}

impl DTO for () {
    fn check(&self) -> Result<()> {
        Ok(())
    }
}

pub async fn init<C: Config + Unpin>(config: C) -> Result<OpServer<C>> {
    let mut server = OpServer {
        config: Arc::new(config),
        user: RwLock::new(BTreeMap::new()),
    };
    home::init(&mut server).await?;
    user::init(&mut server).await?;

    Ok(server)
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
