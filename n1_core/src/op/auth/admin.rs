use std::collections::{BTreeMap, BTreeSet};

use n1_tool::DB;
use serde::Deserialize;

use crate::{
    DTO, OpRequest, Result, errs,
    op::{Administration, Entity, OID_GLOBAL_ENTITY},
};

pub async fn auth_admin_new(db: &impl DB, r: OpRequest<String>) -> Result<u32> {
    r.token.check_isadmin()?;

    let mut entities = r.common.entities.write().await;
    if entities
        .iter()
        .find(|(_, entity)| entity.name() == r.dto)
        .is_some()
    {
        errs::ALREADY_EXIST.push_result(format!("entity {:?} already exist", r.dto))?;
    }

    let id = db.fs_new_entity().await?;
    entities.insert(
        id,
        Entity::Administration(Administration {
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
            errs::FIELD_EMPTY.push_result("field 'id' cannot be zero")?;
        } else if self.name.is_empty() {
            errs::FIELD_EMPTY.push_result("field 'name' cannot be empty")?;
        }
        Ok(())
    }
}
pub async fn auth_admin_edit_name(db: &impl DB, r: OpRequest<EditNameDTO>) -> Result<()> {
    r.token.check_isadmin()?;

    let mut entities = r.common.entities.write().await;
    if entities
        .iter()
        .find(|(_, entity)| entity.name() == r.dto.name)
        .is_some()
    {
        errs::ALREADY_EXIST.push_result(format!("entity {:?} already exist", r.dto.name))?;
    }

    match entities.get_mut(&r.dto.id) {
        Some(Entity::Administration(admin)) => {
            admin.name = r.dto.name;
        }
        Some(_) => {
            errs::INCOHERENT_WITH_STATE.push_result(format!(
                "entity with id {:?} is not an administration",
                r.dto.id
            ))?;
        }
        None => {
            errs::NOT_FOUND.push_result(format!("entity with id {:?} not found", r.dto.id))?;
        }
    }

    // TODO: remove existing page and regenerate.
    save_entities(db, &entities).await?;

    Ok(())
}

async fn save_entities(db: &impl DB, entities: &BTreeMap<u32, Entity>) -> Result<()> {
    let v: Vec<&Entity> = entities.values().collect();
    db.obj_set(0, OID_GLOBAL_ENTITY.into(), v).await
}
