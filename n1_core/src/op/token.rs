use std::{collections::BTreeMap, iter::once};

use serde::Deserialize;

use crate::{
    Result, errs,
    op::{OID_ENTITY_DROPBOX, OID_ENTITY_WIKI, OID_GLOBAL_ENTITY, OID_GLOBAL_HOME, user::Entity},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub uid: u32,
    pub is_admin: bool,
    pub access: [TokenItem; Token::ACCESS_LEN],
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Deserialize)]
pub struct TokenItem {
    pub id: u32,
    pub app: u16,
    pub can_write: bool,
}

impl Token {
    pub const ACCESS_LEN: usize = 63;

    pub fn init() -> Self {
        Self {
            uid: 0,
            is_admin: false,
            access: [TokenItem {
                id: 0,
                app: 0,
                can_write: false,
            }; 63],
        }
    }

    /// Check if the token is authenticated. Return `errs::NO_AUTH` if not authenticated.
    pub fn check_auth(&self) -> Result<()> {
        if self.uid == 0 {
            return Err(errs::NO_AUTH.into());
        }
        Ok(())
    }

    /// Check if the token has admin access.
    pub fn check_admin(&self) -> Result<()> {
        self.check_auth()?;
        if !self.is_admin {
            return Err(errs::FORBIDEN_GLOBAL.into());
        }
        Ok(())
    }

    /// Check if the token has read access to a specific group and app.
    pub fn check_access_read(&self, id: u32, app_id: u16) -> Result<()> {
        self.check_auth()?;
        for item in self.access.iter() {
            if item.id == id && item.app == app_id {
                return Ok(());
            }
        }
        Err(errs::FORBIDEN_GROUP.into())
    }

    /// Check if the token has read access to a specific group and app.
    pub fn check_access_write(&self, id: u32, app_id: u16) -> Result<()> {
        self.check_auth()?;
        for item in self.access.iter() {
            if item.id == id && item.app == app_id && item.can_write {
                return Ok(());
            }
        }
        Err(errs::FORBIDEN_GROUP.into())
    }

    pub fn names<'a>(
        &self,
        entities: &'a BTreeMap<u32, Entity>,
    ) -> impl Iterator<Item = (TokenItem, &'a str)> {
        once(TokenItem {
            id: self.uid,
            app: 0,
            can_write: self.is_admin,
        })
        .chain(self.access.iter().take_while(|item| item.id != 0).copied())
        .map(move |item| (item, entities.get(&item.id).map(Entity::name).unwrap_or("")))
    }

    pub fn test_alice() -> Self {
        let mut access = [TokenItem::default(); 63];
        access[0] = TokenItem {
            id: 1,
            app: OID_GLOBAL_ENTITY,
            can_write: true,
        };
        access[0] = TokenItem {
            id: 1,
            app: OID_GLOBAL_HOME,
            can_write: true,
        };
        access[0] = TokenItem {
            id: 1,
            app: OID_ENTITY_DROPBOX,
            can_write: true,
        };
        access[0] = TokenItem {
            id: 1,
            app: OID_ENTITY_WIKI,
            can_write: true,
        };
        Self {
            uid: 1,
            is_admin: true,
            access,
        }
    }
}

impl Default for Token {
    fn default() -> Self {
        Self {
            uid: 0,
            is_admin: false,
            access: [TokenItem::default(); Token::ACCESS_LEN],
        }
    }
}
