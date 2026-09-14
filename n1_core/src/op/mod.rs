pub mod auth;
mod compo;
pub mod dropbox;
mod dto;
pub mod fs;
pub mod home;
pub mod menu;
mod token;
pub mod user;
pub mod wiki;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::{Arc, RwLock};

use bytes::Bytes;
use n1_tool::{Chunk, Chunks, Config};

use crate::Result;
use crate::op::user::Entity;
pub use dto::*;
pub use token::*;

pub const OID_GLOBAL_ENTITY: u16 = 1;
pub const OID_GLOBAL_HOME: u16 = 3;
pub const OID_ENTITY_DROPBOX: u16 = 4;
pub const OID_ENTITY_WIKI: u16 = 5;

pub struct OpServer<C> {
    pub config: Arc<C>,
    pub entities: RwLock<BTreeMap<u32, Entity>>,
    pub shadow: RwLock<BTreeSet<u32>>,
}

pub struct OpRequest<D: DTO> {
    pub token: Token,
    pub dto: D,
}

impl<C: Config> OpServer<C> {
    pub fn new(config: C) -> Self {
        OpServer {
            config: Arc::new(config),
            entities: RwLock::new(BTreeMap::new()),
            shadow: RwLock::new(BTreeSet::new()),
        }
    }

    pub async fn init(mut self) -> Result<Self> {
        user::init(&mut self).await?;

        home::init(&mut self).await?;
        dropbox::render_public(&self).await?;
        wiki::render_pub(&self).await?;

        Ok(self)
    }
}

/// Indicate that the return type will be retured in JSON.
pub struct Json<T>(pub T);

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
