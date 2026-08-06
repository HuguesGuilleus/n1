mod render_pub;

pub use render_pub::render_pub;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Default)]
pub struct State {
    /// Identifier of the wiki owner
    pub eid: u32,
    /// Shadow id to display wiki.
    pub shadow: u32,
    /// Sorted pages by id.
    pub pages: Vec<Page>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct Page {
    // Page id
    pub id: u32,
    // page end path. Do not contain spaces.
    pub slug: String,
    // Display title
    pub title: String,
    /// Last edition, in second since Epoch.
    pub last_edit: u64,
}
