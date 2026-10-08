mod auth;
pub mod bio;
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
use std::sync::Arc;

use bytes::Bytes;
use n1_tool::{Chunk, Chunks, DB};
use tokio::sync::RwLock;

use crate::Result;
use crate::op::user::Entity;
pub use auth::*;
pub use dto::*;
pub use token::*;

pub const OID_GLOBAL_ENTITY: u16 = 1;
pub const OID_GLOBAL_HOME: u16 = 3;
pub const OID_ENTITY_FS: u16 = 1;
pub const OID_ENTITY_BIO: u16 = 3;
pub const OID_ENTITY_DROPBOX: u16 = 4;
pub const OID_ENTITY_WIKI: u16 = 5;

pub struct OpServer<C> {
    pub db: Arc<C>,
    pub entities: RwLock<BTreeMap<u32, Entity>>,
    pub shadow: RwLock<BTreeSet<u32>>,
}

pub struct OpRequest<D: DTO> {
    pub token: Token,
    pub dto: D,
}

impl<C: DB> OpServer<C> {
    pub fn new(config: C) -> Self {
        OpServer {
            db: Arc::new(config),
            entities: RwLock::new(BTreeMap::new()),
            shadow: RwLock::new(BTreeSet::new()),
        }
    }

    pub async fn init(mut self) -> Result<Self> {
        user::init(&mut self).await?;

        bio::generate_all_pages(&mut self).await?;
        home::init(&mut self).await?;
        dropbox::render_public(&self).await?;
        wiki::render_pub(&self).await?;

        Ok(self)
    }
}

/// Indicate that the return type will be retured in JSON.
pub struct Json<T>(pub T);

pub async fn big<C: DB>(server: &OpServer<C>, _req: OpRequest<()>) -> Result<Chunks<C>> {
    let b1 = Bytes::from_static(b"Hello ");
    let b2 = Bytes::from_static(b"World!\r\n");
    server.db.fs_set(42, 1, b1.clone()).await?;
    server.db.fs_set(42, 2, b2.clone()).await?;

    Ok(Chunks::new(
        server.db.clone(),
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
