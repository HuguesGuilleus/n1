use n1_html::{DirectHTML, H, Html, Q};
use n1_js::js;
use n1_tool::DB;
use serde::Deserialize;

use super::{super::compo, DTO, Entity, OpRequest, Token, TokenItem};
use crate::{Result, errs, front, op::Lang};

#[derive(Debug, Deserialize)]
pub struct LoginDTO {
    name: String,
    password: String,
}
impl DTO for LoginDTO {
    fn check(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(errs::FIELD_NAME.into());
        }
        if self.password.is_empty() {
            return Err(errs::FIELD_PASSWORD.into());
        }
        Ok(())
    }
}
pub async fn login_action(_: &impl DB, r: OpRequest<LoginDTO>) -> Result<Token> {
    let users = r.common.entities.read().await;
    for (_, entity) in users.iter() {
        if let Entity::User(user) = entity {
            if user.name == r.dto.name {
                if user.password == r.dto.password {
                    let mut access = [TokenItem::default(); Token::ACCESS_LEN];
                    access[0..user.access.len()].copy_from_slice(&user.access);
                    return Ok(Token {
                        uid: user.uid,
                        is_admin: user.is_admin,
                        access,
                    });
                } else {
                    return Err(errs::WRONG_LOGIN.into());
                }
            }
        }
    }
    return Err(errs::WRONG_LOGIN.into());
}

pub async fn login_console(_: &impl DB, request: OpRequest<()>) -> Result<String> {
    if request.token.check_auth().is_ok() {
        return Err(errs::ALREADY_CONNECTED.into());
    }

    let lang = request.common.lang;
    Ok([H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + lang.login_title]]
        + [H - "body data-auth=true"
            + compo::goto_main(lang)
            + [H - "header.fh"
                + [H - "h1.bl" + lang.login_title]
                + compo::header_menu(lang)
                + ""]
            + [H - "main.w id=main"
                + [H - "form.act.bb.fv.gap id=form"
                    + [H - "label for=account" + lang.login_account]
                    + [H - "input.bl id=account placeholder='eve'"]
                    + [H - "label for=password" + lang.login_password]
                    + [H - "input.bl type=password id=password placeholder='a5_6yV'"]
                    + [H - "div.fh.gap"
                        + [H - "input.bl type=submit value=" - Q(lang.login_send)]
                        + [H - "span.mv id=out"]
                        + ""]
                    + ""]
                + ""]
            + ""]
        + [H - "script"
            + "const "
            + compo::FETCHER_JS
            + ";"
            + DirectHTML(js!(
                form.onsubmit = _event => _event.preventDefault() | _event.stopPropagation() |
                    fetcher(fetch("/:auth.login", {
                        method: "PUT",
                        body: JSON.stringify({
                            name: account.value,
                            password: password.value,
                        }),
                    }), out).then(_ => {
                        localStorage.setItem("auth", 1);
                        new BroadcastChannel("auth").postMessage(1);
                        location = "/_";
                   });
            ))]
        + ""]
    .render_page())
}

pub fn logout_console(lang: &Lang) -> String {
    [H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + lang.logout_title]]
        + [H - "body"
            + compo::goto_main(lang)
            + [H - "header.fh"
                + [H - "h1.bl" + lang.logout_title]
                + [H - "a.bl.mlauto href=/_user_login" + lang.login_title]
                + ""]
            + [H - "main.w id=main" + lang.logout_info]
            + ""]
        + [H - "script"
            + DirectHTML(js!(
                localStorage.removeItem("auth");
                new BroadcastChannel("auth").postMessage(1);
            ))]
        + ""]
    .render_page()
}
