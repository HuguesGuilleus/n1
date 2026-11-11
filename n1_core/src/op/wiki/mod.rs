//! Public wiki module.
//!
//! Only register user with write access can modify the wiki.
//! But everyone (including anonymous) can view articles.
//!
//! [`WikiState`] contains the meta information about a wiki.
//! The article content is stored as UTF-8 string in the database.

mod action_article;
mod render_priv;
mod render_pub;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::op::Name;
pub use action_article::*;
pub use render_priv::*;
pub use render_pub::render_pub_all;

/// Contain meta information about wiki.
#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct WikiState {
    /// Identifier of the wiki owner.
    pub owner_id: u32,
    /// All aricles meta.
    pub articles: BTreeMap<u32, WikiArticle>,
}

/// Meta data about a wiki article.
#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct WikiArticle {
    // Article id
    pub id: u32,
    // Article end path. Do not contain spaces and no slash.
    pub slug: Name,
    // Display title
    pub title: String,
    /// Last edition, in seconds since Epoch.
    pub last_edit: u64,
}
