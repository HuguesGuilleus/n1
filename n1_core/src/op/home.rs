use bytes::Bytes;
use n1_html::{DirectHTML, H, Html, Q};
use serde::{Deserialize, Serialize};

use super::{DTO, OpRequest, compo};
use super::{OID_GLOBAL_HOME, OpResponse, OpServer};
use crate::{Result, errs, front};
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

impl DTO for HomeState {
    fn check(&self) -> Result<()> {
        if self.title.is_empty() {
            return Err(errs::FIELD_TITLE);
        }
        if self.desc.is_empty() {
            return Err(errs::FIELD_DESC);
        }
        if self.content.is_empty() {
            return Err(errs::FIELD_CONTENT);
        }
        Ok(())
    }
}

pub async fn init(server: &OpServer<impl Config>) -> Result<()> {
    let mut state = server
        .config
        .obj_fetch(0, OID_GLOBAL_HOME)
        .await
        .unwrap_or_else(|_| HomeState::default());

    let mut edit = false;
    if state.title.is_empty() {
        edit = true;
        state.title = "Welcome".to_string();
    }
    if state.desc.is_empty() {
        edit = true;
        state.desc = "your new drive!".to_string();
    }
    if state.content.is_empty() {
        edit = true;
        state.content = "Welcome...".to_string();
    }

    if edit {
        server.config.obj_store(0, OID_GLOBAL_HOME, &state).await?;
    }

    server
        .config
        .page_add("/", mime::HTML, render_pub(&state))
        .await?;

    Ok(())
}

fn render_pub(home: &HomeState) -> Bytes {
    Bytes::from_owner(
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
                + compo::header(false, "Accueil")
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
            + ""]
        .render_page(),
    )
}

pub async fn edit<C: Config>(
    server: &OpServer<C>,
    r: &OpRequest<HomeState>,
) -> Result<OpResponse<C>> {
    r.token.access_global(super::TokenLevel::Admin)?;

    server
        .config
        .page_add("/", mime::HTML, render_pub(&r.dto))
        .await?;

    server.config.obj_store(0, OID_GLOBAL_HOME, &r.dto).await?;

    Ok(OpResponse::Ok)
}

pub async fn console<C: Config>(server: &OpServer<C>, r: &OpRequest<()>) -> Result<OpResponse<C>> {
    r.token.access_global(super::TokenLevel::Admin)?;
    let state = server.config.obj_fetch(0, OID_GLOBAL_HOME).await?;
    Ok(OpResponse::Bytes(mime::HTML, render_console(&state)))
}

fn render_console(state: &HomeState) -> Bytes {
    Bytes::from_owner([H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + "Modification de l'accueil"]]
         + [H - "body"
            + compo::header(true ,  "Modification de l'accueil" )
            + [H - "main.w"
                    + [H - "div.act.bb.fv.gap"
                        + [H - "div.g2.gap"
                            + [H - "label for=_title" + "Titre:"]
                            + [H - "input.bg id=_title value=" - Q(&state.title)]
                        + ""]
                        + [H - "label for=_desc" + "Description:"]
                        + [H - "div.act.bg contenteditable id=_desc" + &state.desc]
                        + [H - "label for=_content" + "Contenu de la première page, utiliser des lignes vides pour séparer les paragraphes:"]
                        + [H - "pre.act.bg contenteditable id=_content" +&state.content]
                        + [H - "button.bl onclick=send()" +"Envoyer"]
                    + ""]
                    +[H - "script" + DirectHTML(r#"const send=()=>{
                        fetch("/_home/", {
                            method: "PUT",
                            body: JSON.stringify({
                                title: _title.value,
                                desc: _desc.innerText ,
                                content: _content.innerText ,
                            }),
                        }).then(r=>r);
                    }"#)]
                + ""]
            + ""]
        + ""]
    .render_page())
}
