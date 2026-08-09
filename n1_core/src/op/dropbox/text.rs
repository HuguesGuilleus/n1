use n1_tool::Config;
use serde::Deserialize;

use crate::{
    DTO, OpRequest, OpServer, Result, errs,
    op::{EntityAndObjectDTO, OID_ENTITY_DROPBOX, TokenLevel, dropbox::State},
};

#[derive(Debug, PartialEq, Deserialize)]
pub struct IDandString {
    pub eid: u32,
    pub str: String,
}
impl DTO for IDandString {
    fn check(&self) -> Result<()> {
        if self.eid == 0 {
            return Err(errs::FIELD_ENTITY.into());
        }
        if self.str.is_empty() {
            return Err(errs::FIELD_STR.into());
        }
        Ok(())
    }
}
pub async fn text_add(server: &OpServer<impl Config>, r: OpRequest<IDandString>) -> Result<()> {
    let mut state: State = server
        .config
        .obj_fetch(r.dto.eid, OID_ENTITY_DROPBOX)
        .await?;

    state.texts.push((state.texts_inc, r.dto.str));
    state.texts_inc += 1;

    server
        .config
        .obj_store(r.dto.eid, OID_ENTITY_DROPBOX, &state)
        .await?;

    Ok(())
}

pub async fn text_rm(
    server: &OpServer<impl Config>,
    r: OpRequest<EntityAndObjectDTO>,
) -> Result<()> {
    if r.token.uid != r.dto.eid {
        r.token.access_group(r.dto.eid, TokenLevel::Write)?;
    }

    let mut state: State = server
        .config
        .obj_fetch(r.dto.eid, OID_ENTITY_DROPBOX)
        .await?;

    let index = state
        .texts
        .binary_search_by(|(id, _)| (*id).cmp(&r.dto.oid))
        .map_err(|_| errs::NOT_FOUND)?;
    state.texts.remove(index);

    server
        .config
        .obj_store(r.dto.eid, OID_ENTITY_DROPBOX, &state)
        .await?;

    Ok(())
}

#[tokio::test]
async fn text() -> Result<()> {
    let server = crate::init_dev().await?;

    text_add(
        &server,
        OpRequest {
            token: crate::op::Token::test_alice(),
            dto: IDandString {
                eid: 1,
                str: "Hello World!".to_string(),
            },
        },
    )
    .await
    .unwrap();

    text_rm(
        &server,
        OpRequest {
            token: crate::op::Token::test_alice(),
            dto: EntityAndObjectDTO { eid: 1, oid: 0 },
        },
    )
    .await
    .unwrap();

    Ok(())
}
