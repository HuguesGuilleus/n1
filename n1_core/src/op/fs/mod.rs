mod console_direct;
mod op_direct;

use std::collections::BTreeMap;

pub use console_direct::*;
use n1_tool::{Chunk, DB};
pub use op_direct::*;
use serde::{Deserialize, Serialize};

use crate::{Common, Result, op::OID_ENTITY_FS};

const CHUNK_LEN: usize = 65536;

/// A file system of on user or group.
///
/// The data must be a tree.
/// Invalid 2 file/directory with same name.
/// The directory 0 is the root directory.
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct FSState {
    pub entries: BTreeMap<u32, Entry>,

    pub inc: u32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub enum Entry {
    Directory {
        parent: u32,
        children: BTreeMap<String, u32>,
    },
    File {
        parent: u32,
        chunks: Vec<Chunk>,
    },
}

impl Default for FSState {
    fn default() -> Self {
        let mut items = BTreeMap::new();
        items.insert(
            0,
            Entry::Directory {
                parent: 0,
                children: BTreeMap::new(),
            },
        );
        Self {
            entries: items,
            inc: 1,
        }
    }
}

pub async fn init(db: &impl DB, common: &Common) -> Result<()> {
    let entities = common.entities.read().await;
    for &id in entities.keys() {
        let fs = common.fs.get(id).await?;
        let mut fs = fs.write().await;
        *fs = db.obj_get(id, OID_ENTITY_FS as u32).await?;
    }
    Ok(())
}
