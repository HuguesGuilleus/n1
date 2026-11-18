use std::{collections::BTreeMap, ops::Deref};

use n1_tool::{Chunk, DB, chuncks_len, read_to_chunks};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncRead;

use crate::{
    DTO, Error, ErrorStep, OpRequest, Result,
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
            ErrorStep::FieldZero { name: "owner" }.result()?
        }
        check_path(&self.path)?;
        Ok(())
    }
}
fn check_path(path: &str) -> Result<()> {
    let path = prepare_path(path);
    if path.is_empty() {
        ErrorStep::FieldEmpty { name: "path" }.result()?
    }
    for part in path.split("/") {
        match part.trim() {
            "" => ErrorStep::PathPartEmpty.result()?,
            "." | ".." => ErrorStep::PathPartDots.result()?,
            _ => {}
        }
    }
    Ok(())
}
/// Remove leading and trailing slashes from a path string.
fn prepare_path(path: &str) -> &str {
    let path = path.strip_prefix("/").unwrap_or(path);
    let path = path.strip_suffix("/").unwrap_or(path);
    path
}

/// Make a new directory. Path: `:fs.direct.mkdir`
///
/// If can recursive create parent directories.
/// If directory already exist, do nothing and return last directory id.
pub async fn fs_direct_mkdir(db: &impl DB, r: OpRequest<MakeDTO>) -> Result<u32> {
    make(db, r, true).await
}

/// Make a new file. Path: `:fs.direct.mkfile`
///
/// If file already exist, do nothing.
/// Create recursive parent directory if it do not exist.
pub async fn fs_direct_mkfile(db: &impl DB, r: OpRequest<MakeDTO>) -> Result<u32> {
    make(db, r, false).await
}

async fn make(db: &impl DB, r: OpRequest<MakeDTO>, is_dir: bool) -> Result<u32> {
    r.token.check_access_write(r.dto.owner, OID_ENTITY_FS)?;

    let (path, last) = if is_dir {
        (prepare_path(&r.dto.path), "")
    } else {
        let path = prepare_path(&r.dto.path);
        path.rsplit_once("/").unwrap_or(("", path))
    };

    let guard = r.common.fs.get(r.dto.owner).await?;
    let mut fs = guard.write().await;

    let mut id = fs_direct_make(&mut fs, path, r.dto.owner)?;
    if !last.is_empty() {
        while fs.inc == 0 || fs.entries.contains_key(&fs.inc) {
            fs.inc += 1;
        }
        let new_id = fs.inc;
        let mut used_new_id = false;

        let children = get_children(&mut fs, r.dto.owner, id)?;
        match children.get(last) {
            Some(&child_id) => match fs.entries.get(&child_id) {
                Some(Entry::File { .. }) => {
                    id = child_id;
                }
                Some(Entry::Directory { .. }) => {
                    return Err(ErrorStep::PathPartIsDirectory { id: child_id }.into());
                }
                None => {
                    return Err(Error::new([
                        ErrorStep::IncoherentFS {
                            owner: r.dto.owner,
                            id: child_id,
                        },
                        ErrorStep::NotFoundObject {
                            owner: r.dto.owner,
                            object: child_id,
                            type_name: "fs-item",
                        },
                    ]));
                }
            },
            None => {
                used_new_id = true;
                children.insert(last.to_string(), new_id);
                fs.entries.insert(
                    new_id,
                    Entry::File {
                        parent: id,
                        chunks: Vec::new(),
                    },
                );
                id = new_id;
            }
        }
        if used_new_id {
            fs.inc += 1;
        }
    }

    db.obj_set(r.dto.owner, OID_ENTITY_FS as u32, fs.deref())
        .await?;

    Ok(id)
}
/// Get or create the directory path and return its ID.
fn fs_direct_make(fs: &mut FSState, path: &str, owner: u32) -> Result<u32> {
    let mut id = 0;
    for part in path
        .split("/")
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        while fs.inc == 0 || fs.entries.contains_key(&fs.inc) {
            fs.inc += 1;
        }
        let new_id = fs.inc;
        let mut used_new_id = false;

        let children = get_children(fs, owner, id)?;
        if let Some(child) = children.get(part) {
            id = *child;
        } else {
            used_new_id = true;
            children.insert(part.to_string(), new_id);
            fs.entries.insert(
                new_id,
                Entry::Directory {
                    parent: id,
                    children: BTreeMap::new(),
                },
            );
            id = new_id;
        }
        if used_new_id {
            fs.inc += 1;
        }
    }

    Ok(id)
}
/// Get the children of a directory by its ID.
fn get_children(fs: &mut FSState, owner: u32, id: u32) -> Result<&mut BTreeMap<String, u32>> {
    let children = match fs.entries.get_mut(&id) {
        Some(Entry::Directory { children, .. }) => children,
        Some(Entry::File { .. }) => return Err(ErrorStep::PathPartIsFile { id }.into()),
        None => {
            return Err(Error::new([
                ErrorStep::IncoherentFS { owner, id },
                ErrorStep::NotFoundObject {
                    owner,
                    object: id,
                    type_name: "fs-item",
                },
            ]));
        }
    };
    Ok(children)
}

pub async fn fs_direct_set(
    db: &impl DB,
    r: OpRequest<EntityAndObjectDTO>,
    content: impl AsyncRead + Unpin + Send,
) -> Result<(u32, Vec<Chunk>)> {
    r.token.check_access_write(r.dto.owner, OID_ENTITY_FS)?;

    let new_chunks = read_to_chunks::<CHUNK_LEN>(db, r.dto.owner, content).await?;

    let fs_guard = r.common.fs.get(r.dto.owner).await?;
    let mut fs = fs_guard.write().await;

    let entry = fs
        .entries
        .get_mut(&r.dto.object)
        .ok_or_else(|| ErrorStep::NotFoundObject {
            owner: r.dto.owner,
            object: r.dto.object,
            type_name: "fs-entry",
        })?;
    match entry {
        Entry::File { chunks, .. } => {
            for chunk in chunks.iter() {
                db.fs_rm_object(r.dto.owner, chunk.oid).await?;
            }
            *chunks = new_chunks;
        }
        Entry::Directory { .. } => {
            ErrorStep::PathPartIsDirectory { id: r.dto.object }.result()?;
        }
    }

    db.obj_set(r.dto.owner, OID_ENTITY_FS as u32, fs.deref())
        .await?;

    Ok((0, Vec::new()))
}

pub async fn fs_direct_get(
    _: &impl DB,
    r: OpRequest<EntityAndObjectDTO>,
    _: impl AsyncRead + Unpin + Send,
) -> Result<(u32, Vec<Chunk>)> {
    r.token.check_access_read(r.dto.owner, OID_ENTITY_FS)?;

    let fs_guard = r.common.fs.get(r.dto.owner).await?;
    let fs = fs_guard.read().await;

    let entry = fs
        .entries
        .get(&r.dto.object)
        .ok_or(ErrorStep::NotFoundObject {
            owner: r.dto.owner,
            object: r.dto.object,
            type_name: "file",
        })?;

    match entry {
        Entry::File { chunks, .. } => {
            return Ok((r.dto.owner, chunks.clone()));
        }
        Entry::Directory { .. } => {
            return Err(ErrorStep::PathPartIsDirectory { id: r.dto.object }.into());
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct MoveDTO {
    pub owner: u32,
    pub object: u32,
    pub path: String,
}
impl DTO for MoveDTO {
    fn check(&self) -> Result<()> {
        if self.owner == 0 {
            ErrorStep::FieldZero { name: "owner" }.result()?
        } else if self.object == 0 {
            ErrorStep::FieldZero { name: "object" }.result()?
        }
        check_path(&self.path)?;
        Ok(())
    }
}
pub async fn fs_direct_move(db: &impl DB, r: OpRequest<MoveDTO>) -> Result<()> {
    r.token.check_access_write(r.dto.owner, OID_ENTITY_FS)?;

    let owner = r.dto.owner;
    let fs_guard = r.common.fs.get(owner).await?;
    let mut fs = fs_guard.write().await;

    // Get entry and its parent ID
    let entry = fs
        .entries
        .get_mut(&r.dto.object)
        .ok_or(ErrorStep::NotFoundObject {
            owner,
            object: r.dto.object,
            type_name: "fs-entry",
        })?;
    let old_parent_id = match entry {
        Entry::Directory { parent, .. } => *parent,
        Entry::File { parent, .. } => *parent,
    };

    // Remove item from the parent children
    match fs.entries.get_mut(&old_parent_id) {
        Some(Entry::Directory {
            parent: _,
            children,
        }) => {
            children.retain(|_, &mut id| id != r.dto.object);
        }
        Some(Entry::File { .. }) => {
            return Err(Error::new([
                ErrorStep::IncoherentFS {
                    owner,
                    id: old_parent_id,
                },
                ErrorStep::PathPartIsFile { id: old_parent_id },
            ]));
        }
        None => {
            return Err(Error::new([ErrorStep::IncoherentFS {
                owner,
                id: old_parent_id,
            }]));
        }
    }

    // Insert the entry into the new parent's children
    let path = prepare_path(&r.dto.path);
    let (path, name) = path.rsplit_once('/').unwrap_or(("", path));
    let new_parent = fs_direct_make(&mut fs, path, owner)?;
    let new_children = get_children(&mut fs, owner, new_parent)?;
    new_children.insert(name.to_string(), r.dto.object);
    let entry = fs
        .entries
        .get_mut(&r.dto.object)
        .ok_or(ErrorStep::NotFoundObject {
            owner,
            object: r.dto.object,
            type_name: "fs-entry",
        })?;
    match entry {
        Entry::Directory { parent, .. } => *parent = new_parent,
        Entry::File { parent, .. } => *parent = new_parent,
    }

    db.obj_set(owner, OID_ENTITY_FS as u32, fs.deref()).await?;

    Ok(())
}

pub async fn fs_direct_rm(db: &impl DB, r: OpRequest<EntityAndObjectDTO>) -> Result<()> {
    r.token.check_access_write(r.dto.owner, OID_ENTITY_FS)?;

    let fs_guard = r.common.fs.get(r.dto.owner).await?;
    let mut fs = fs_guard.write().await;

    let mut root = None; // get only the first parent encountered
    let mut ids_to_remove = vec![r.dto.object];
    while let Some(id) = ids_to_remove.pop() {
        match fs.entries.remove(&id) {
            Some(Entry::Directory { parent, children }) => {
                root = root.or(Some(parent));
                ids_to_remove.extend(children.values().copied());
            }
            Some(Entry::File { parent, chunks }) => {
                root = root.or(Some(parent));
                for id in chunks.iter().map(|chunk| chunk.oid) {
                    db.fs_rm_object(r.dto.owner, id).await?;
                }
            }
            None => {}
        }
    }

    if let Some(parent) = root {
        match fs.entries.get_mut(&parent) {
            Some(Entry::Directory {
                parent: _,
                children,
            }) => {
                children.retain(|_, &mut id| id != r.dto.object);
            }
            Some(Entry::File { .. }) => {
                return Err(Error::new([
                    ErrorStep::IncoherentFS {
                        owner: r.dto.owner,
                        id: parent,
                    },
                    ErrorStep::PathPartIsDirectory { id: parent },
                ]));
            }
            None => {}
        }
    }

    db.obj_set(r.dto.owner, OID_ENTITY_FS as u32, fs.deref())
        .await?;

    Ok(())
}

pub async fn fs_direct_resolve(_: &impl DB, r: OpRequest<MakeDTO>) -> Result<u32> {
    r.token.check_access_read(r.dto.owner, OID_ENTITY_FS)?;

    fn walk(fs: &FSState, owner: u32, id: u32, part: &str) -> Result<u32> {
        let part = part.trim();
        let entry = fs.entries.get(&id).ok_or_else(|| {
            Error::new([
                ErrorStep::IncoherentFS { owner, id },
                ErrorStep::NotFoundObject {
                    owner,
                    object: id,
                    type_name: "fs-entry",
                },
            ])
        })?;
        let children = match entry {
            Entry::Directory { children, .. } => children,
            Entry::File { .. } => {
                return Err(ErrorStep::PathPartIsFile { id }.into());
            }
        };
        children.get(part).copied().ok_or_else(|| {
            ErrorStep::NotFoundObject {
                owner,
                object: id,
                type_name: "fs-entry",
            }
            .into()
        })
    }

    let fs_guard = r.common.fs.get(r.dto.owner).await?;
    let fs = fs_guard.read().await;

    let mut id = 0;
    for part in prepare_path(&r.dto.path).split("/") {
        id = walk(&fs, r.dto.owner, id, part)?;
    }

    Ok(id)
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Branch {
    id: u32,
    is_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<BTreeMap<String, Branch>>,
}
pub async fn fs_direct_tree(_: &impl DB, r: OpRequest<u32>) -> Result<Branch> {
    r.token.check_access_read(r.dto, OID_ENTITY_FS)?;

    fn new_branch(fs: &FSState, owner: u32, id: u32) -> Result<Branch> {
        let entry = fs.entries.get(&id).ok_or_else(|| {
            Error::new([
                ErrorStep::IncoherentFS { owner, id },
                ErrorStep::NotFoundObject {
                    owner,
                    object: id,
                    type_name: "fs-entry",
                },
            ])
        })?;
        match entry {
            Entry::Directory { children, .. } => {
                let mut branch_children = BTreeMap::new();
                for (name, &child_id) in children {
                    branch_children.insert(name.clone(), new_branch(fs, owner, child_id)?);
                }
                Ok(Branch {
                    id,
                    is_dir: true,
                    content: None,
                    children: Some(branch_children),
                })
            }
            Entry::File { chunks, .. } => Ok(Branch {
                id,
                is_dir: false,
                content: Some(chuncks_len(chunks)),
                children: None,
            }),
        }
    }

    let fs_guard = r.common.fs.get(r.dto).await?;
    let fs = fs_guard.read().await;

    new_branch(&fs, r.dto, 0)
}

#[tokio::test]
async fn test() {
    use crate::op::{Common, Token, TokenItem};
    use n1_tool::DBMemory;
    use std::sync::Arc;

    let db = DBMemory::new();
    let common = Arc::new(Common::new());
    let token = Token {
        uid: 1,
        is_admin: true,
        access: [TokenItem::default(); Token::ACCESS_LEN],
    };
    fs_direct_mkfile(
        &db,
        OpRequest {
            common: common.clone(),
            token: token.clone(),
            dto: MakeDTO {
                owner: 1,
                path: "/dir/sub/file.txt".to_string(),
            },
        },
    )
    .await
    .unwrap();

    {
        let guard = common.fs.get(1).await.unwrap();
        let state = guard.read().await;
        assert_eq!(
            &FSState {
                entries: BTreeMap::from([
                    (
                        0,
                        Entry::Directory {
                            parent: 0,
                            children: BTreeMap::from([("dir".to_string(), 1)]),
                        }
                    ),
                    (
                        1,
                        Entry::Directory {
                            parent: 0,
                            children: BTreeMap::from([("sub".to_string(), 2)]),
                        }
                    ),
                    (
                        2,
                        Entry::Directory {
                            parent: 1,
                            children: BTreeMap::from([("file.txt".to_string(), 3)]),
                        }
                    ),
                    (
                        3,
                        Entry::File {
                            parent: 2,
                            chunks: Vec::new(),
                        }
                    )
                ]),
                inc: 4
            },
            state.deref(),
        );
    }

    // Resolve
    assert_eq!(
        fs_direct_resolve(
            &db,
            OpRequest {
                common: common.clone(),
                token: token.clone(),
                dto: MakeDTO {
                    owner: 1,
                    path: "/dir/sub/".to_string(),
                },
            },
        )
        .await
        .unwrap(),
        2
    );
    assert_eq!(
        fs_direct_resolve(
            &db,
            OpRequest {
                common: common.clone(),
                token: token.clone(),
                dto: MakeDTO {
                    owner: 1,
                    path: "/dir/sub/file.txt".to_string(),
                },
            },
        )
        .await
        .unwrap(),
        3
    );

    // Move
    fs_direct_move(
        &db,
        OpRequest {
            common: common.clone(),
            token: token.clone(),
            dto: MoveDTO {
                owner: 1,
                object: 3,
                path: "file.txt".to_string(),
            },
        },
    )
    .await
    .unwrap();
    fs_direct_move(
        &db,
        OpRequest {
            common: common.clone(),
            token: token.clone(),
            dto: MoveDTO {
                owner: 1,
                object: 2,
                path: "dir/sub/subsub".to_string(),
            },
        },
    )
    .await
    .unwrap();
    {
        let guard = common.fs.get(1).await.unwrap();
        let state = guard.read().await;
        assert_eq!(
            &FSState {
                entries: BTreeMap::from([
                    (
                        0,
                        Entry::Directory {
                            parent: 0,
                            children: BTreeMap::from([
                                ("dir".to_string(), 1),
                                ("file.txt".to_string(), 3),
                            ]),
                        }
                    ),
                    (
                        1,
                        Entry::Directory {
                            parent: 0,
                            children: BTreeMap::from([("sub".to_string(), 4)]),
                        }
                    ),
                    (
                        2,
                        Entry::Directory {
                            parent: 4,
                            children: BTreeMap::from([]),
                        }
                    ),
                    (
                        3,
                        Entry::File {
                            parent: 0,
                            chunks: Vec::new(),
                        }
                    ),
                    (
                        4,
                        Entry::Directory {
                            parent: 1,
                            children: BTreeMap::from([("subsub".to_string(), 2)]),
                        }
                    )
                ]),
                inc: 5
            },
            state.deref(),
        );
    }
}
