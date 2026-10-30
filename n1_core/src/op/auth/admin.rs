use std::collections::{BTreeMap, BTreeSet};

use n1_tool::DB;

use crate::{
    OpRequest, Result, errs,
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

async fn save_entities(db: &impl DB, entities: &BTreeMap<u32, Entity>) -> Result<()> {
    let v: Vec<&Entity> = entities.values().collect();
    db.obj_set(0, OID_GLOBAL_ENTITY.into(), v).await
}
