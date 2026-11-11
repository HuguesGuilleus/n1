pub mod auth;
pub mod bio;
mod compo;
mod dto;
pub mod fs;
pub mod home;
mod lang;
mod lang_en;
mod lang_fr;
pub mod menu;
mod name;
mod token;
pub mod wiki;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

use n1_tool::DB;
use tokio::sync::Mutex;
use tokio::sync::RwLock;

use crate::ErrorStep;
use crate::Result;
pub use auth::*;
pub use dto::*;
pub use lang::*;
pub use lang_en::LANG_EN;
pub use lang_fr::LANG_FR;
pub use name::Name;
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
    pub lang: &'static Lang,

    pub entities: RwLock<BTreeMap<u32, Entity>>,
    pub shadow: RwLock<BTreeSet<u32>>,
    pub fs: LockMap<fs::FSState>,
    pub wiki: LockMap<wiki::WikiState>,
}

pub struct LockMap<T>(Mutex<BTreeMap<u32, Arc<RwLock<T>>>>);

impl<T: Default> LockMap<T> {
    /// Get data of a specific user
    pub async fn get(&self, owner: u32) -> Result<Arc<RwLock<T>>> {
        let mut guard = self.0.lock().await;
        let arc = match guard.get(&owner) {
            Some(arc) => arc.clone(),
            None => {
                let arc = Arc::new(RwLock::new(T::default()));
                guard.insert(owner, arc.clone());
                arc
            }
        };
        Ok(arc)
    }
}

impl Common {
    pub fn new() -> Self {
        Common {
            lang: &LANG_EN,

            entities: RwLock::new(BTreeMap::new()),
            shadow: RwLock::new(BTreeSet::new()),
            fs: LockMap(Mutex::new(BTreeMap::new())),
            wiki: LockMap(Mutex::new(BTreeMap::new())),
        }
    }

    pub async fn init(&mut self, db: &impl DB) -> Result<()> {
        auth::init(db, self).await?;

        bio::generate_all_pages(db, self).await?;
        fs::init(db, self).await?;
        home::init(db, self).await?;
        wiki::render_pub_all(db, self).await?;

        Ok(())
    }

    pub async fn get_entity(&self, id: u32) -> Result<Entity> {
        let entities = self.entities.read().await;
        let entity = entities
            .get(&id)
            .ok_or(ErrorStep::NotFoundEntity { entity: id })?;
        if entity.is_none() {
            ErrorStep::ExpectRealEntity { entity: id }.result()?
        }
        Ok(entity.clone())
    }

    pub async fn get_entity_name(&self, id: u32) -> Result<String> {
        let entities = self.entities.read().await;
        let entity = entities
            .get(&id)
            .ok_or(ErrorStep::NotFoundEntity { entity: id })?;
        if entity.is_none() {
            ErrorStep::ExpectRealEntity { entity: id }.result()?
        }
        Ok(entity.name().to_string())
    }
}
