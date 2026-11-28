use std::collections::{BTreeMap, BTreeSet};

use n1_tool::DB;
use serde::Deserialize;

use crate::{
    op::{Entity, Organization, OID_GLOBAL_ENTITY},
    Error, ErrorStep, OpRequest, Result, DTO,
};

pub async fn auth_org_new(db: &impl DB, r: OpRequest<String>) -> Result<u32> {
    r.token.check_isadmin()?;

    let mut entities = r.common.entities.write().await;
    if entities
        .iter()
        .find(|(_, entity)| entity.name() == r.dto)
        .is_some()
    {
        ErrorStep::AlreadyExist {}.result()?;
    }

    let id = db.fs_new_owner().await?;
    entities.insert(
        id,
        Entity::Organization(Organization {
            id,
            name: r.dto,
            entities: BTreeSet::new(),
            default_access: Vec::new(),
        }),
    );
    save_entities(db, &entities).await?;

    Ok(id)
}

#[derive(Debug, Deserialize)]
pub struct EditNameDTO {
    id: u32,
    name: String,
}
impl DTO for EditNameDTO {
    fn check(&self) -> Result<()> {
        if self.id == 0 {
            ErrorStep::FieldZero { name: "id" }.result()?;
        } else if self.name.is_empty() {
            ErrorStep::FieldZero { name: "name" }.result()?;
        }
        Ok(())
    }
}
pub async fn auth_org_edit_name(db: &impl DB, r: OpRequest<EditNameDTO>) -> Result<()> {
    r.token.check_isadmin()?;

    let mut entities = r.common.entities.write().await;
    if entities
        .iter()
        .find(|(_, entity)| entity.name() == r.dto.name)
        .is_some()
    {
        ErrorStep::AlreadyExist {}.result()?;
    }

    match entities.get_mut(&r.dto.id) {
        Some(Entity::Organization(org)) => {
            org.name = r.dto.name;
        }
        Some(_) => {
            ErrorStep::ExpectOrganizationEntity { entity: r.dto.id }.result()?;
        }
        None => {
            ErrorStep::NotFoundEntity { entity: r.dto.id }.result()?;
        }
    }

    // TODO: remove existing page and regenerate.
    save_entities(db, &entities).await?;

    Ok(())
}

async fn save_entities(db: &impl DB, entities: &BTreeMap<u32, Entity>) -> Result<()> {
    let v: Vec<&Entity> = entities.values().collect();
    db.obj_set(0, OID_GLOBAL_ENTITY.into(), v)
        .await
        .map_err(Error::from)
}
