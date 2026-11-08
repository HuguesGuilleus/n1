//! Home page configuration and generation.
//!
//! The access is administrator reserved.

use bytes::Bytes;
use n1_js::js;
use serde::{Deserialize, Serialize};

use super::{DTO, OID_GLOBAL_HOME, OpRequest, compo};
use crate::{Common, ErrorStep, Result, front, op::Lang};
use n1_html::{DirectHTML, H, Html, Q};
use n1_tool::{DB, mime};

/// The State and DTO for home.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct HomeState {
    /// Title of the root page
    title: String,
    /// Description, used for meta page information.
    desc: String,
    /// Content, each line in a separate paragraph
    content: String,
}

/// Initialise default data and generate home page.
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

/// Check all field are not empty.
impl DTO for HomeState {
    fn check(&self) -> Result<()> {
        if self.title.is_empty() {
            return Err(ErrorStep::FieldEmpty { name: "title" }.into());
        }
        if self.desc.is_empty() {
            return Err(ErrorStep::FieldEmpty { name: "desc" }.into());
        }
        if self.content.is_empty() {
            return Err(ErrorStep::FieldEmpty { name: "content" }.into());
        }
        Ok(())
    }
}
/// API to change home page data, and regenerate it.
pub async fn json_edit(db: &impl DB, r: OpRequest<HomeState>) -> Result<()> {
    r.token.check_isadmin()?;
    db.obj_set(0, OID_GLOBAL_HOME as u32, &r.dto).await?;
    db.page_set("/", mime::HTML, render_pub(r.common.lang, &r.dto))
        .await?;
    Ok(())
}

/// Render the home page.
fn render_pub(lang: &Lang, home: &HomeState) -> Bytes {
    Bytes::from_owner(
        [H - "html lang=" - lang.code
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

/// Render home page console.
pub async fn home_console(db: &impl DB, r: OpRequest<()>) -> Result<String> {
    r.token.check_isadmin()?;
    let state: HomeState = db.obj_get(0, OID_GLOBAL_HOME as u32).await?;

    let lang = r.common.lang;
    Ok([H - "html lang=" - lang.code
        + [H - "head" + front::HEAD + [H - "title" + lang.home_title]]
        + [H - "body"
            + [H - "header.fh"
                + [H - "h1.bl" + lang.home_title]
                + compo::header_menu_logged(lang)]
            + [H - "main.w"
                + [H - "div.act.fv.gap"
                    + [H - "div.g2.gap"
                        + [H - "label for=title" + lang.home_edit_title]
                        + [H - "input.bl id=title value=" - Q(&state.title)]
                        + ""]
                    + [H - "label for=desc" + lang.home_edit_description]
                    + [H - "textarea.act.bl.norh id=desc" + &state.desc]
                    + [H - "label for=content" + lang.home_page_content]
                    + [H - "textarea.act.bl.norh id=content rows="
                        - (state.content.split("\n").count() + 3)
                        + &state.content]
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
                                desc: desc.value,
                                content: content.value,
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
