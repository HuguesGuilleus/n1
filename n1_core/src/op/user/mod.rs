pub mod login;

use n1_tool::{AtomicError, Config, mime};
use serde::Deserialize;

use super::{DTO, OpRequest, OpServer};
use crate::{
    Result,
    op::{OID_GLOBAL_ENTITY, TokenLevel},
};

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub enum Entity {
    None(u32),
    User(User),
    Group(Group),
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct User {
    pub uid: u32,
    pub name: String,
    pub password: String,
    pub global: TokenLevel,
    /// Access for this group
    pub groups_array: [(u32, TokenLevel); 5],
    pub groups_vec: Vec<(u32, TokenLevel)>,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Group {
    pub gid: u32,
    pub name: String,
    pub users: Vec<(u32, TokenLevel)>,
}

pub async fn init<C: Config>(serv: &mut OpServer<C>) -> Result<()> {
    serv.config
        .page_add("/_login", mime::HTML, login::render())
        .await?;

    // Load entities
    let entities: Vec<Entity> = serv.config.obj_fetch(0, OID_GLOBAL_ENTITY).await?;
    let entities_map = serv.entities.get_mut().map_err(AtomicError::from)?;
    entities.into_iter().for_each(|entity| {
        entities_map.insert(
            match entity {
                Entity::None(id) => id,
                Entity::User(User { uid, .. }) => uid,
                Entity::Group(Group { gid, .. }) => gid,
            },
            entity,
        );
    });

    Ok(())
}

impl Entity {
    pub fn id(&self) -> u32 {
        match self {
            Entity::None(id) => *id,
            Entity::User(user) => user.uid,
            Entity::Group(group) => group.gid,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Entity::None(_) => "~",
            Entity::User(user) => &user.name,
            Entity::Group(group) => &group.name,
        }
    }
}

impl Default for Entity {
    fn default() -> Self {
        Entity::None(0)
    }
}

impl User {
    pub fn groups(&self) -> impl Iterator<Item = (u32, TokenLevel)> {
        self.groups_array
            .into_iter()
            .chain(self.groups_vec.iter().copied())
            .filter(|(id, _)| *id != 0)
    }
}
