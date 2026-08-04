mod compo;
pub mod dropbox;
pub mod home;
pub mod menu;
mod token;
pub mod user;

use std::sync::{Arc, RwLock};
use std::{collections::BTreeMap, fmt::Debug};

use bytes::Bytes;
use n1_tool::{Chunk, Chunks, Config};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::op::user::Entity;
use crate::{Result, errs};
pub use token::*;

pub const OID_GLOBAL_ENTITY: u32 = 1;
pub const OID_GLOBAL_HOME: u32 = 3;
pub const OID_ENTITY_DROPBOX: u32 = 4;

pub struct OpServer<C> {
    pub config: Arc<C>,
    pub entities: RwLock<BTreeMap<u32, Entity>>,
}

pub struct OpRequest<D: DTO> {
    pub token: Token,
    pub dto: D,
}

pub trait DTO: DeserializeOwned + Debug {
    fn check(&self) -> Result<()>;
}

impl DTO for () {
    fn check(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct ID(u32);

impl DTO for ID {
    fn check(&self) -> Result<()> {
        if self.0 == 0 {
            return Err(errs::FIELD_ID);
        }
        Ok(())
    }
}

pub async fn init<C: Config + Unpin>(config: C) -> Result<OpServer<C>> {
    let mut server = OpServer {
        config: Arc::new(config),
        entities: RwLock::new(BTreeMap::new()),
    };
    home::init(&mut server).await?;
    user::init(&mut server).await?;

    Ok(server)
}

pub async fn big<C: Config>(server: &OpServer<C>, _req: OpRequest<()>) -> Result<Chunks<C>> {
    let b1 = Bytes::from_static(b"Hello ");
    let b2 = Bytes::from_static(b"World!\r\n");
    server.config.fs_set(42, 1, b1.clone()).await?;
    server.config.fs_set(42, 2, b2.clone()).await?;

    Ok(Chunks::new(
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
    ))
}
