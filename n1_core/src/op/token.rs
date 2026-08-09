use std::{collections::BTreeMap, iter::once};

use serde::{Deserialize, Serialize};

use crate::{Result, errs, op::user::Entity};

/// A parsed token with all user access.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Token {
    /// User ID.
    /// No auth if `id == 0`
    pub uid: u32,
    /// Level access to global resources.
    pub global: TokenLevel,
    /// Access for this group
    pub groups_array: [(u32, TokenLevel); 5],
    pub groups_vec: Vec<(u32, TokenLevel)>,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Serialize, Deserialize, Default)]
pub enum TokenLevel {
    /// Zero access
    #[default]
    None = 0,
    /// The user can see the data.
    Read = 1,
    // The user can write data
    Write = 2,
    /// The user can remove the group and manage user.
    Admin = 3,
}

impl Token {
    /// Token to init operation with all priviledge.
    pub fn init() -> Self {
        Self {
            uid: 1,
            global: TokenLevel::Admin,
            groups_array: [(0, TokenLevel::Admin); 5],
            groups_vec: Vec::with_capacity(0),
        }
    }

    /// Check authentification. Return `errs::NO_AUTH` if no authentification.
    pub fn is_auth(&self) -> Result<()> {
        if self.uid == 0 {
            return Err(errs::NO_AUTH.into());
        }
        Ok(())
    }
    /// Check if the token permit to access this global value.
    pub fn access_global(&self, level: TokenLevel) -> Result<()> {
        self.is_auth()?;
        if self.global < level {
            return Err(errs::FORBIDEN_GLOBAL.into());
        }
        Ok(())
    }
    /// Check if the token permit this value for a spectific group.
    pub fn access_group(&self, gid: u32, level: TokenLevel) -> Result<()> {
        self.is_auth()?;
        let iter = self
            .groups_array
            .into_iter()
            .chain(self.groups_vec.iter().copied())
            .filter(|&(token_group, _)| token_group != 0);
        for (token_group, token_level) in iter {
            if token_group == gid {
                if token_level < level {
                    return Err(errs::FORBIDEN_GROUP.into());
                }
                return Ok(());
            }
        }
        Err(errs::FORBIDEN_OUTSIDE.into())
    }

    pub fn groups(&self) -> impl Iterator<Item = (u32, TokenLevel)> {
        once((self.uid, TokenLevel::Admin))
            .chain(self.groups_array.into_iter())
            .chain(self.groups_vec.iter().copied())
            .filter(|(gid, _)| *gid != 0)
    }

    pub fn groups_name<'a>(
        &self,
        entities: &'a BTreeMap<u32, Entity>,
    ) -> impl Iterator<Item = (u32, TokenLevel, &'a str)> {
        once((self.uid, TokenLevel::Admin))
            .chain(self.groups_array.into_iter())
            .chain(self.groups_vec.iter().copied())
            .filter(|(gid, _)| *gid != 0)
            .map(move |(eid, level)| {
                (
                    eid,
                    level,
                    entities.get(&eid).map(Entity::name).unwrap_or(""),
                )
            })
    }

    /// Create a super admin user.
    pub fn test_alice() -> Self {
        Self {
            uid: 1,
            global: TokenLevel::Admin,
            groups_array: [
                (10, TokenLevel::Write),
                (0, TokenLevel::Read),
                (0, TokenLevel::Read),
                (0, TokenLevel::Read),
                (0, TokenLevel::Read),
            ],
            groups_vec: Vec::with_capacity(0),
        }
    }

    // Create simple user
    pub fn test_bob() -> Self {
        Self {
            uid: 1,
            global: TokenLevel::Read,
            groups_array: [
                (10, TokenLevel::Write),
                (0, TokenLevel::Read),
                (0, TokenLevel::Read),
                (0, TokenLevel::Read),
                (0, TokenLevel::Read),
            ],
            groups_vec: Vec::with_capacity(0),
        }
    }
}

#[test]
fn test_token_level_order() {
    assert!(TokenLevel::None < TokenLevel::Read);
    assert!(TokenLevel::None < TokenLevel::Write);
    assert!(TokenLevel::None < TokenLevel::Admin);

    assert!(TokenLevel::Read < TokenLevel::Write);
    assert!(TokenLevel::Read < TokenLevel::Admin);
    assert!(TokenLevel::Write < TokenLevel::Admin);
}

impl TokenLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Read => "read",
            Self::Write => "write",
            Self::Admin => "admin",
        }
    }
}
