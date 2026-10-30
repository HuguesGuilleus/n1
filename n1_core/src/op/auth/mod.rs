pub mod admin;
mod auth_check;
mod auth_test;
pub mod login;

pub use admin::*;
pub use auth_check::*;
pub use auth_test::*;

use std::collections::BTreeSet;

use n1_tool::DB;
use serde::{Deserialize, Serialize};

use crate::{
    Common, DTO, OpRequest, Result,
    op::{OID_GLOBAL_ENTITY, Token, TokenItem},
};

#[derive(Debug, PartialEq, Deserialize, Clone, Serialize)]
pub enum Entity {
    Administration(Administration),
    Group(Group),
    None(u32),
    User(User),
}

#[derive(Debug, PartialEq, Deserialize, Clone, Serialize)]
pub struct Administration {
    pub id: u32,
    pub name: String,
    pub entities: BTreeSet<u32>,
    pub default_access: Vec<TokenItem>,
}

#[derive(Debug, PartialEq, Deserialize, Clone, Serialize)]
pub struct User {
    pub uid: u32,
    pub name: String,
    pub password: String,
    pub is_admin: bool,
    /// Access for this group
    pub access: Vec<TokenItem>,
}

#[derive(Debug, PartialEq, Deserialize, Clone, Serialize)]
pub struct Group {
    pub gid: u32,
    pub name: String,
    pub users: Vec<TokenItem>,
}

pub async fn init(db: &impl DB, common: &mut Common) -> Result<()> {
    // Load entities
    let entities: Vec<Entity> = db.obj_get(0, OID_GLOBAL_ENTITY as u32).await?;
    let entities_map = common.entities.get_mut();
    entities.into_iter().for_each(|entity| {
        entities_map.insert(
            match entity {
                Entity::Administration(Administration { id, .. }) => id,
                Entity::Group(Group { gid, .. }) => gid,
                Entity::None(id) => id,
                Entity::User(User { uid, .. }) => uid,
            },
            entity,
        );
    });

    Ok(())
}

impl Entity {
    pub fn id(&self) -> u32 {
        match self {
            Entity::Administration(admin) => admin.id,
            Entity::Group(group) => group.gid,
            Entity::None(id) => *id,
            Entity::User(user) => user.uid,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Entity::Administration(admin) => &admin.name,
            Entity::Group(group) => &group.name,
            Entity::User(user) => &user.name,
            Entity::None(_) => "~",
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            Entity::None(_) => true,
            _ => false,
        }
    }
}

impl Default for Entity {
    fn default() -> Self {
        Entity::None(0)
    }
}

impl User {
    pub fn groups(&self) -> impl Iterator<Item = u32> {
        let mut groups = [0u32; Token::ACCESS_LEN];
        let mut groups_inc = 0;

        let mut previous = 0;
        for TokenItem { id, .. } in &self.access {
            if *id != previous {
                previous = *id;
                groups[groups_inc] = *id;
                groups_inc += 1;
            }
        }

        groups
            .into_iter()
            .take_while(|&id| id != 0)
            .filter(|&gid| gid != self.uid)
    }
}
