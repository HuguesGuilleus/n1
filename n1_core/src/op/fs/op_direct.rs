use std::ops::Deref;

use n1_tool::{Chunk, DB, read_to_chunks};
use serde::Deserialize;
use tokio::io::AsyncRead;

use crate::{
    DTO, OpRequest, Result, errs,
    op::{
        EntityAndObjectDTO, OID_ENTITY_FS,
        fs::{CHUNK_LEN, Entry, FSState},
    },
};

#[derive(Debug, Deserialize)]
pub struct MakeDTO {
    pub owner: u32,
    pub path: String,
}
impl DTO for MakeDTO {
    fn check(&self) -> Result<()> {
        if self.owner == 0 {
            errs::FIELD_ID_ZERO.push_result("field 'owner'")?;
        }

        let path = self.path.strip_prefix("/").unwrap_or(&self.path);
        let path = path.strip_suffix("/").unwrap_or(path);
        if path.is_empty() {
            errs::FIELD_EMPTY.push_result("field 'path'")?;
        }
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
/// Make a new directory. Path: `:fs.direct.mkdir`
///
/// If can recursive create parent directories.
/// If directory already exist, do nothing and return last directory id.
pub async fn mkdir(db: &impl DB, r: OpRequest<MakeDTO>) -> Result<u32> {
    make(db, r, true).await
}

/// Make a new file. Path: `:fs.direct.mkfile`
///
/// If file already exist, do nothing.
/// Create recursive parent directory if it do not exist.
pub async fn mkfile(db: &impl DB, r: OpRequest<MakeDTO>) -> Result<u32> {
    make(db, r, false).await
}

async fn make(db: &impl DB, r: OpRequest<MakeDTO>, is_dir: bool) -> Result<u32> {
    r.token.check_access_write(r.dto.owner, OID_ENTITY_FS)?;

    fn add_directory(fs: &mut FSState, id: &mut u32, part: &str) -> Result<()> {
        let (_, dir) = fs
            .dirs
            .get_mut(&id)
            .ok_or_else(|| errs::DATA_INCOERENT.push(format!("not found {} directory", id)))?;
        match dir.iter().filter(|&entry| entry.name == part).next() {
            Some(entry) if entry.is_dir => {
                *id = entry.id;
            }
            Some(_) => {
                errs::ALREADY_EXIST.push_result(format!("{} is a regular file", part))?;
            }
            None => {
                fs.increment += 1;
                let new_id = fs.increment;
                dir.push(Entry {
                    id: new_id,
                    name: part.to_string(),
                    is_dir: true,
                });
                fs.dirs.insert(new_id, (*id, Vec::new()));
                *id = new_id;
            }
        }
        Ok(())
    }

    let path = r.dto.path.strip_prefix("/").unwrap_or(&r.dto.path);
    let path = path.strip_suffix("/").unwrap_or(path);
    let (path, last) = if !is_dir {
        path.rsplit_once("/").unwrap_or(("", path))
    } else {
        (path, "")
    };
    let fs_guard = r.common.fs.get(r.dto.owner).await?;
    let mut fs = fs_guard.write().await;
    let mut id = 0;

    if !path.is_empty() {
        // Possible, because "/new_file.txt"
        for part in path.split("/").map(str::trim) {
            add_directory(&mut fs, &mut id, part)?;
        }
    }

    if !is_dir {
        let fs: &mut FSState = &mut fs;
        let (_, dir) = fs
            .dirs
            .get_mut(&id)
            .ok_or_else(|| errs::DATA_INCOERENT.push(format!("not found {} directory", id)))?;
        match dir.iter().filter(|&entry| entry.name == last).next() {
            Some(entry) if !entry.is_dir => {
                id = entry.id;
            }
            Some(_) => {
                errs::ALREADY_EXIST.push_result(format!("{} is a directory", last))?;
            }
            None => {
                fs.increment += 1;
                let new_id = fs.increment;
                dir.push(Entry {
                    id: new_id,
                    name: last.to_string(),
                    is_dir: false,
                });
                fs.files.insert(new_id, (id, Vec::new()));
                id = new_id;
            }
        }
    }

    db.obj_set(r.dto.owner, OID_ENTITY_FS as u32, fs.deref())
        .await?;

    Ok(id)
}

pub async fn fs_direct_set(
    db: &impl DB,
    r: OpRequest<EntityAndObjectDTO>,
    content: impl AsyncRead + Unpin + Send,
) -> Result<(u32, Vec<Chunk>)> {
    r.token.check_access_write(r.dto.owner, OID_ENTITY_FS)?;

    let chunks = read_to_chunks::<CHUNK_LEN>(db, r.dto.owner, content).await?;

    let fs_guard = r.common.fs.get(r.dto.owner).await?;
    let mut fs = fs_guard.write().await;

    let file = fs
        .files
        .get_mut(&r.dto.object)
        .ok_or_else(|| errs::NOT_FOUND.push(format!("file {}", r.dto.object)))?;
    file.1 = chunks;

    db.obj_set(r.dto.owner, OID_ENTITY_FS as u32, fs.deref())
        .await?;

    Ok((0, Vec::new()))
}
