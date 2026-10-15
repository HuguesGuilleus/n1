mod action_article;
mod render_priv;
mod render_pub;

use std::collections::BTreeMap;

pub use action_article::*;
pub use render_priv::*;
pub use render_pub::render_pub;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct State {
    /// Identifier of the wiki owner.
    pub owner_id: u32,
    /// Sorted pages by id.
    pub articles: BTreeMap<u32, Article>,
}

/// Meta data about a wiki article.
#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct Article {
    // Article id
    pub article_id: u32,
    // Article end path. Do not contain spaces and no slash.
    pub slug: String,
    // Display title
    pub title: String,
    /// Last edition, in second since Epoch.
    pub last_edit: u64,
}
