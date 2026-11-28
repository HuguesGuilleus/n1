pub mod admin_org;
mod auth_check;
mod console;
pub mod login;

pub use admin_org::*;
pub use auth_check::*;
pub use console::*;

use std::collections::BTreeSet;

use n1_tool::DB;
use serde::{Deserialize, Serialize};

use crate::{
    op::{Token, TokenItem, OID_GLOBAL_ENTITY},
    Common, OpRequest, Result, DTO,
};

#[derive(Debug, PartialEq, Deserialize, Clone, Serialize)]
pub enum Entity {
    /// A mega group containing multiple groups and users.
    Organization(Organization),
    /// One group.
    Group(Group),
    /// One user.
    User(User),
    /// Old entity id reserved.
    None(u32),
}

#[derive(Debug, PartialEq, Deserialize, Clone, Serialize)]
pub struct Organization {
    pub id: u32,
    pub name: String,
    pub entities: BTreeSet<u32>,
    pub default_access: Vec<TokenItem>,
}

#[derive(Debug, PartialEq, Deserialize, Clone, Serialize)]
pub struct Group {
    pub gid: u32,
    pub name: String,
    pub users: Vec<TokenItem>,
    pub organization: u32,
}

#[derive(Debug, PartialEq, Deserialize, Clone, Serialize)]
pub struct User {
    pub uid: u32,
    pub name: String,
    pub password: String,
    pub is_admin: bool,
    pub organization: u32,
    pub groups: Vec<TokenItem>,
}

/// Load entities informations.
pub async fn init(db: &impl DB, common: &Common) -> Result<()> {
    let entities: Vec<Entity> = db.obj_get(0, OID_GLOBAL_ENTITY as u32).await?;
    let mut entities_map = common.entities.write().await;
    entities.into_iter().for_each(|entity| {
        entities_map.insert(
            match entity {
                Entity::Organization(Organization { id, .. }) => id,
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
            Entity::Organization(org) => org.id,
            Entity::Group(group) => group.gid,
            Entity::None(id) => *id,
            Entity::User(user) => user.uid,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Entity::Organization(org) => &org.name,
            Entity::Group(group) => &group.name,
            Entity::User(user) => &user.name,
            Entity::None(_) => "~",
        }
    }

    pub fn is_org(&self) -> bool {
        match self {
            Entity::Organization(_) => true,
            _ => false,
        }
    }

    pub fn is_group(&self) -> bool {
        match self {
            Entity::Group(_) => true,
            _ => false,
        }
    }

    pub fn is_user(&self) -> bool {
        match self {
            Entity::User(_) => true,
            _ => false,
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
        for TokenItem { id, .. } in &self.groups {
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
