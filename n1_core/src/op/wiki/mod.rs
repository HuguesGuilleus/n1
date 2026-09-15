mod action_article;
mod render_priv;
mod render_pub;

pub use action_article::*;
pub use render_priv::*;
pub use render_pub::render_pub;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Default)]
pub struct State {
    /// Identifier of the wiki owner
    pub eid: u32,
    /// Shadow id to display wiki.
    /// If the code is zero, this wiki is public.
    pub shadow: u32,
    /// Sorted pages by id.
    pub articles: Vec<Article>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct Article {
    // Page id
    pub oid: u32,
    // page end path. Do not contain spaces.
    pub slug: String,
    // Display title
    pub title: String,
    /// Last edition, in second since Epoch.
    pub last_edit: u64,
    /// The page content.
    pub content: String,
}
