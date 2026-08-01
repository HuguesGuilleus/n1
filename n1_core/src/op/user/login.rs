use bytes::Bytes;
use n1_html::{DirectHTML, H, Html};
use n1_tool::Config;
use serde::Deserialize;

use super::{DTO, OpRequest, OpServer};
use crate::{
    Result, errs, front,
    op::{OpResponse, Token, compo, user::Entity},
};

#[derive(Debug, Deserialize)]
pub struct LoginDTO {
    name: String,
    password: String,
}

impl DTO for LoginDTO {
    fn check(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(errs::FIELD_NAME);
        }
        if self.password.is_empty() {
            return Err(errs::FIELD_PASSWORD);
        }
        Ok(())
    }
}

pub async fn login<C: Config>(
    serv: &OpServer<C>,
    r: &OpRequest<LoginDTO>,
) -> Result<OpResponse<C>> {
    let users = serv.entities.read()?;
    for (_, entity) in users.iter() {
        if let Entity::User(user) = entity {
            if user.name == r.dto.name {
                if user.password == r.dto.password {
                    return Ok(OpResponse::Token(Token {
                        uid: user.uid,
                        global: user.global,
                        groups_array: user.groups_array,
                        groups_vec: user.groups_vec.clone(),
                    }));
                } else {
                    return Err(errs::WRONG_LOGIN);
                }
            }
        }
    }
    return Err(errs::WRONG_LOGIN);
}

pub fn render() -> Bytes {
    Bytes::from_owner(
        [H - "html lang=fr"
            + [H - "head" + front::HEAD + [H - "title" + "Connexion"]]
            + [H - "body"
                + compo::header(false, "Connexion")
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
                    r#"const send=()=>{
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
