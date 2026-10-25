mod op_direct;

use std::collections::BTreeMap;

use n1_tool::Chunk;
pub use op_direct::*;
use serde::{Deserialize, Serialize};

/// A file system of on user or group.
///
/// The data must be a tree.
/// Invalid 2 file/directory with same name.
/// The directory 0 is the root directory.
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct FSState {
    /// Parent id and directory content.
    pub dirs: BTreeMap<u32, (u32, Vec<Entry>)>,
    /// Parent id and file content.
    pub files: BTreeMap<u32, (u32, Vec<Chunk>)>,
    /// Used to generate identifiers of file or directory.
    /// Increment before use as new id.
    pub increment: u32,
}

/// File or directory meta information stored in [`FSState::dirs`]
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Entry {
    /// The file system identifier of this entry.
    pub id: u32,
    /// The item name.
    pub name: String,
    /// It's a regular file or a directory?
    pub is_dir: bool,
}

impl Default for FSState {
    fn default() -> Self {
        let mut dirs = BTreeMap::new();
        dirs.insert(0, (0, Vec::new()));
        Self {
            dirs,
            files: BTreeMap::new(),
            increment: 0,
        }
    }
}
