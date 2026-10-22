mod op_direct;

use std::collections::BTreeMap;

use n1_tool::Chunk;
pub use op_direct::*;
use serde::{Deserialize, Serialize};

/// A file system of on user or group.
///
/// The data should be a tree.
/// We make verification to avoid duplicate entri with 2 same name.
/// The directory 0 is the root directory.
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct FSState {
    /// Directory content.
    pub dirs: BTreeMap<u32, Vec<Entry>>,
    /// File content.
    pub files: BTreeMap<u32, Vec<Chunk>>,
    /// Used to generate identifiers of filee or directory.
    /// Increment before use as new id.
    pub increment: u32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Entry {
    /// The file system identifier of this entry.
    /// Zero value indicate
    pub id: u32,
    /// The item name.
    pub name: String,
    /// It's a regular file or a directory?
    pub is_dir: bool,
}

impl Default for FSState {
    fn default() -> Self {
        let mut dirs = BTreeMap::new();
        dirs.insert(0, Vec::new());
        Self {
            dirs,
            files: BTreeMap::new(),
            increment: 0,
        }
    }
}
