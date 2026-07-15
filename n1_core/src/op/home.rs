use bytes::Bytes;
use n1_html::{DirectHTML, H, Html};
use serde::{Deserialize, Serialize};

use super::{OID_GLOBAL_HOME, OpServer};
use crate::{Result, front};
use n1_tool::{Config, mime};

/// The State and DTO for home.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct HomeState {
    /// Title of the root page
    title: String,
    /// Each paragraph
    desc: String,
    /// Content, each line in a separate paragraph
    content: String,
}

pub async fn init(server: &OpServer<impl Config>) -> Result<()> {
    let mut state = server
        .config
        .obj_fetch(0, OID_GLOBAL_HOME)
        .await
        .unwrap_or_else(|_| HomeState::default());

    if state.title.is_empty() {
        state.title = "Welcome".to_string();
    }
    if state.desc.is_empty() {
        state.desc = "your new drive!".to_string();
    }
    if state.content.is_empty() {
        state.content = "Welcome...".to_string();
    }

    server
        .config
        .page_add("/", mime::HTML, Bytes::from_owner(render_pub(&state)))
        .await?;

    Ok(())
}

pub fn render_pub(home: &HomeState) -> String {
    [H - "html lang=fr"
        + [H - "head"
            + front::HEAD
            + [H - "title" + &home.title]
            + [H - "meta name=description "
                - DirectHTML("content=\"")
                - &home.desc
                - DirectHTML("\"")]
            + ""]
        + [H - "body"
            + [H - "header.fh"
                + [H - "a.bl href=/_" + "///"]
                + [H - "a.bl id=login href=/login" + "Connexion"]
                + [H - "a.bl.act.mlauto id=logout href=/logout" + "Déconnexion"]
                + ""]
            + [H - "main.w"
                + [H - "h1" + &home.title]
                + (|| {
                    home.content
                        .lines()
                        .filter(|line| !line.is_empty())
                        .map(|line| [H - "p" + line.trim()])
                })
                + ""]
            + ""]
        + [H - "script" + DirectHTML(r#"(localStorage.getItem("isauth")?login:logout).hidden=!0"#)]
        + ""]
    .render_page()
}
