use bytes::Bytes;
use n1_js::js;
use serde::{Deserialize, Serialize};

use super::{DTO, OID_GLOBAL_HOME, OpRequest, compo};
use crate::{Common, Result, errs, front, op::Lang};
use n1_html::{DirectHTML, H, Html, Q};
use n1_tool::{DB, mime};

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
            return Err(errs::FIELD_TITLE.into());
        }
        if self.desc.is_empty() {
            return Err(errs::FIELD_DESC.into());
        }
        if self.content.is_empty() {
            return Err(errs::FIELD_CONTENT.into());
        }
        Ok(())
    }
}

pub async fn init(db: &impl DB, common: &Common) -> Result<()> {
    let mut state = db
        .obj_get(0, OID_GLOBAL_HOME as u32)
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
        db.obj_set(0, OID_GLOBAL_HOME as u32, &state).await?;
    }

    db.page_set("/", mime::HTML, render_pub(common.lang, &state))
        .await?;

    Ok(())
}

pub async fn json_edit(db: &impl DB, r: OpRequest<HomeState>) -> Result<()> {
    r.token.check_isadmin()?;
    db.obj_set(0, OID_GLOBAL_HOME as u32, &r.dto).await?;
    db.page_set("/", mime::HTML, render_pub(r.common.lang, &r.dto))
        .await?;
    Ok(())
}

fn render_pub(lang: &Lang, home: &HomeState) -> Bytes {
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
                + [H - "header.fh" + [H - "h1.bl" + &home.title] + compo::header_menu(lang)]
                + [H - "main.w"
                    + (|| {
                        home.content
                            .lines()
                            .filter(|line| !line.is_empty())
                            .map(|line| [H - "p" + line.trim()])
                    })
                    + ""]
                + ""]
            + [H - "script" + compo::LOGIN_JS]
            + ""]
        .render_page(),
    )
}

pub async fn home_console(db: &impl DB, r: OpRequest<()>) -> Result<String> {
    r.token.check_isadmin()?;
    let state: HomeState = db.obj_get(0, OID_GLOBAL_HOME as u32).await?;

    let lang = r.common.lang;
    Ok([H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + lang.home_title]]
        + [H - "body"
            + [H - "header.fh"
                + [H - "h1.bl" + lang.home_title]
                + compo::header_menu_logged(lang)]
            + [H - "main.w"
                + [H - "div.act.fv.gap"
                    + [H - "div.g2.gap"
                        + [H - "label for=title" + lang.home_edite_title]
                        + [H - "input.bl id=title value=" - Q(&state.title)]
                        + ""]
                    + [H - "label for=desc" + lang.home_edite_description]
                    + [H - "div.act.bl contenteditable id=desc" + &state.desc]
                    + [H - "label for=content" + lang.home_page_content]
                    + [H - "pre.act.bl contenteditable id=content" + &state.content]
                    + [H - "div.fh.gap"
                        + [H - "button.bl onclick=send()" + lang.common_save]
                        + [H - "a.bl href=/ " + lang.home]]
                    + [H - "div id=out"]
                    + ""]
                + [H - "script"
                    + "const "
                    + compo::FETCHER_JS
                    + ","
                    + DirectHTML(js!(
                        send = _ => fetcher(fetch("/:home/", {
                            method: "PUT",
                            body: JSON.stringify({
                                title: title.value,
                                desc: desc .innerText,
                                content: content.innerText,
                            }),
                        }), out);
                    ))
                    + compo::LOGIN_JS_CONNECTED
                    + ""]
                + ""]
            + ""]
        + [H - "script" + compo::LOGIN_JS]
        + ""]
    .render_page())
}
