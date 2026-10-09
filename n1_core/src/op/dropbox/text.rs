use n1_tool::DB;
use serde::Deserialize;

use crate::{
    DTO, OpRequest, Result, errs,
    op::{EntityAndObjectDTO, OID_ENTITY_DROPBOX, dropbox::State},
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
pub async fn text_add(db: &impl DB, r: OpRequest<IDandString>) -> Result<()> {
    let mut state: State = db.obj_get(r.dto.eid, OID_ENTITY_DROPBOX as u32).await?;

    state.texts.push((state.texts_inc, r.dto.str));
    state.texts_inc += 1;

    db.obj_set(r.dto.eid, OID_ENTITY_DROPBOX as u32, &state)
        .await?;

    Ok(())
}

pub async fn text_rm(db: &impl DB, r: OpRequest<EntityAndObjectDTO>) -> Result<()> {
    if r.token.uid != r.dto.eid {
        r.token.check_access_write(r.dto.eid, OID_ENTITY_DROPBOX)?;
    }

    let mut state: State = db.obj_get(r.dto.eid, OID_ENTITY_DROPBOX as u32).await?;

    let index = state
        .texts
        .binary_search_by(|(id, _)| (*id).cmp(&r.dto.oid))
        .map_err(|_| errs::NOT_FOUND)?;
    state.texts.remove(index);

    db.obj_set(r.dto.eid, OID_ENTITY_DROPBOX as u32, &state)
        .await?;

    Ok(())
}

#[tokio::test]
async fn text() -> Result<()> {
    let (db, common) = crate::init_dev().await?;
    let common = std::sync::Arc::new(common);

    text_add(
        &db,
        OpRequest {
            common: common.clone(),
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
        &db,
        OpRequest {
            common: common.clone(),
            token: crate::op::Token::test_alice(),
            dto: EntityAndObjectDTO { eid: 1, oid: 0 },
        },
    )
    .await
    .unwrap();

    Ok(())
}
