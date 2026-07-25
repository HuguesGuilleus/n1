pub mod login;

use n1_tool::{Config, mime};
use serde::Deserialize;

use super::{DTO, OpRequest, OpServer};
use crate::{
    Result,
    op::{OID_GLOBAL_USER, TokenLevel},
};

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

pub async fn init<C: Config>(serv: &mut OpServer<C>) -> Result<()> {
    serv.config
        .page_add("/_login", mime::HTML, login::render())
        .await?;

    let users: Vec<User> = serv.config.obj_fetch(0, OID_GLOBAL_USER).await?;
    let users_map = serv.user.get_mut()?;
    for u in users {
        users_map.insert(u.uid, u.clone());
    }

    users_map.insert(
        101,
        User {
            uid: 101,
            name: "eve".to_string(),
            password: "56".to_string(),
            global: TokenLevel::Admin,
            groups_array: [
                (201, TokenLevel::Admin),
                (0, TokenLevel::None),
                (0, TokenLevel::None),
                (0, TokenLevel::None),
                (0, TokenLevel::None),
            ],
            groups_vec: Vec::with_capacity(0),
        },
    );

    Ok(())
}
