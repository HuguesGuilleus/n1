use std::ops::Deref;

use n1_tool::DB;
use serde::Deserialize;

use crate::{
    DTO, OpRequest, Result, errs,
    op::{OID_ENTITY_FS, fs::Entry},
};

#[derive(Debug, Deserialize)]
pub struct MkdirDTO {
    pub owner: u32,
    pub path: String,
}
impl DTO for MkdirDTO {
    fn check(&self) -> Result<()> {
        if self.owner == 0 {
            errs::FIELD_ID_ZERO.push_result("field 'owner'")?;
        }

        if self.path.is_empty() {
            errs::FIELD_EMPTY.push_result("field 'path'")?;
        }
        let path = self.path.strip_prefix("/").unwrap_or(&self.path);
        let path = path.strip_suffix("/").unwrap_or(path);
        for part in path.split("/") {
            match part.trim() {
                "" => errs::WRONG_PATH.push_result("empty path part")?,
                "." | ".." => errs::WRONG_PATH.push_result("invalid part '.' or '..' ")?,
                _ => {}
            }
        }

        Ok(())
    }
}
/// Make a new directory.
/// If can recursive create parent directories.
/// If directory already exist, do nothing and return last directory id.
pub async fn mkdir(db: &impl DB, r: OpRequest<MkdirDTO>) -> Result<u32> {
    r.token.check_access_write(r.dto.owner, OID_ENTITY_FS)?;

    let path = r.dto.path.strip_prefix("/").unwrap_or(&r.dto.path);
    let path = path.strip_suffix("/").unwrap_or(path);

    let fs_guard = r.common.fs.get(r.dto.owner).await?;
    let mut fs = fs_guard.write().await;
    let mut increment = fs.increment;
    let mut id = 0;
    for part in path.split("/").map(str::trim) {
        let dir = fs
            .dirs
            .get_mut(&id)
            .ok_or_else(|| errs::DATA_INCOERENT.push(format!("not found {} directory ", id)))?;
        match dir.iter().filter(|&entry| entry.name == part).next() {
            Some(entry) if entry.is_dir => {
                id = entry.id;
            }
            Some(_) => {
                errs::ALREADY_EXIST.push_result(format!("{} is a file", part))?;
            }
            None => {
                increment += 1;
                let new_id = increment;
                dir.push(Entry {
                    id: new_id,
                    name: part.to_string(),
                    is_dir: true,
                });
                fs.dirs.insert(new_id, Vec::new());
                id = new_id;
            }
        }
    }
    fs.increment = increment;

    db.obj_set(r.dto.owner, OID_ENTITY_FS as u32, fs.deref())
        .await?;

    Ok(id)
}
