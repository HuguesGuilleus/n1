use n1_tool::Config;
use serde::Deserialize;

use crate::{
    DTO, OpRequest, OpServer, Result, errs,
    op::{OID_ENTITY_DROPBOX, TokenLevel, dropbox::State},
};

#[derive(Debug, PartialEq, Deserialize)]
pub struct IDandString {
    pub eid: u32,
    pub str: String,
}
impl DTO for IDandString {
    fn check(&self) -> Result<()> {
        if self.eid == 0 {
            return Err(errs::FIELD_ENTITY);
        }
        if self.str.is_empty() {
            return Err(errs::FIELD_STR);
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

#[derive(Debug, PartialEq, Deserialize)]
pub struct IDAndEntity {
    pub eid: u32,
    pub oid: u32,
}
impl DTO for IDAndEntity {
    fn check(&self) -> Result<()> {
        println!("check") ;
         if self.eid == 0 {
            return Err(errs::FIELD_ENTITY);
        }
        if self.oid == 0 {
            return Err(errs::FIELD_OID);
        }
        Ok(())
    }
}
pub async fn text_rm(server: &OpServer<impl Config>, r: OpRequest<IDAndEntity>) -> Result<()> {
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
    let mut server = super::super::init(n1_tool::ConfigMemoryMutex::new()).await?;
    crate::init_dev(&mut server).await?;

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
            dto: IDAndEntity { eid: 1, oid: 0 },
        },
    )
    .await
    .unwrap();

    Ok(())
}
