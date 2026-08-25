mod op;

use std::collections::BTreeMap;

use n1_tool::Chunk;
pub use op::*;
use serde::{Deserialize, Serialize};

/// A file tree.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Clone)]
pub struct FsysState {
    pub dirs: Vec<BTreeMap<String, Entry>>,
    pub dirs_increment: u32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum Entry {
    Dir(DirEntry),
    File(FileEntry),
}
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct DirEntry {
    /// The file system identifier of this entry.
    /// Zero value indicate
    id: u32,
    /// The parent id to go backward in exploration.
    parent: u32,
    /// The directory name.
    name: String,
}
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct FileEntry {
    /// The file system identifier of this entry.
    id: u32,
    /// The file name.
    name: String,
    /// The content chunks informations.
    content: Vec<Chunk>,
}
