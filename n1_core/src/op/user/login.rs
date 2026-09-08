use bytes::Bytes;
use n1_html::{DirectHTML, H, Html};
use n1_tool::{AtomicError, Config};
use serde::Deserialize;

use super::{DTO, OpRequest, OpServer};
use crate::{
    Result, errs, front,
    op::{Token, compo, user::Entity},
};

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

pub async fn login(serv: &OpServer<impl Config>, r: OpRequest<LoginDTO>) -> Result<Token> {
    let users = serv.entities.read().map_err(AtomicError::from)?;
    for (_, entity) in users.iter() {
        if let Entity::User(user) = entity {
            if user.name == r.dto.name {
                if user.password == r.dto.password {
                    return Ok(Token {
                        uid: user.uid,
                        global: user.global,
                        groups_array: user.groups_array,
                        groups_vec: user.groups_vec.clone(),
                    });
                } else {
                    return Err(errs::WRONG_LOGIN.into());
                }
            }
        }
    }
    return Err(errs::WRONG_LOGIN.into());
}

pub fn render() -> Bytes {
    Bytes::from_owner(
        [H - "html lang=fr"
            + [H - "head" + front::HEAD + [H - "title" + "Connexion"]]
            + [H - "body"
                + compo::header("Connexion")
                + [H - "main.w"
                    + [H - "div.act.bb.fv.gap"
                        + [H - "label for=_name" + "Nom du compte:"]
                        + [H - "input.bl id=_name placeholder='eve'"]
                        + [H - "label for=_password" + "Mot de passe:"]
                        + [H - "input.bl type=password id=_password placeholder='a5_6yV'"]
                        + [H - "button.bl onclick=send()" + "Envoyer"]
                        + ""]
                    + ""]
                + ""]
            + [H - "script"
                + DirectHTML(
                    r#"
                    _password.addEventListener("keydown", event => event.key == "Enter" && send());
                    const send=()=>{
                        fetch("/:login", {
                            method: "PUT",
                            body: JSON.stringify({
                                name: _name.value,
                                password: _password.value,
                            }),
                        }).then(r=>r.ok && (location="/_"));
                    }"#,
                )]
            + ""]
        .render_page(),
    )
}
