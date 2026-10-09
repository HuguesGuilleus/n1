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

use n1_tool::DB;
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

pub struct OpRequest<D: DTO> {
    pub common: Arc<Common>,
    pub token: Token,
    pub dto: D,
}

/// Common value shared between all operations.
pub struct Common {
    pub entities: RwLock<BTreeMap<u32, Entity>>,
    pub shadow: RwLock<BTreeSet<u32>>,
}

impl Common {
    pub fn new() -> Self {
        Common {
            entities: RwLock::new(BTreeMap::new()),
            shadow: RwLock::new(BTreeSet::new()),
        }
    }

    pub async fn init(&mut self, db: &impl DB) -> Result<()> {
        user::init(db, self).await?;

        bio::generate_all_pages(db, self).await?;
        home::init(db).await?;
        dropbox::render_public(db, self).await?;
        wiki::render_pub(db, self).await?;

        Ok(())
    }
}

// pub async fn big<B: DB>(db: &B, _: OpRequest<()>) -> Result<Chunks<B>> {
//     let b1 = Bytes::from_static(b"Hello ");
//     let b2 = Bytes::from_static(b"World!\r\n");
//     db.fs_set(42, 1, b1.clone()).await?;
//     db.fs_set(42, 2, b2.clone()).await?;

//     Ok(Chunks::new(
//         db,
//         42,
//         &[
//             Chunk {
//                 len: b1.len(),
//                 oid: 1,
//             },
//             Chunk {
//                 len: b2.len(),
//                 oid: 2,
//             },
//         ],
//     ))
// }
