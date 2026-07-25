use bytes::Bytes;

use crate::*;
use n1_html::{H, Html};
use n1_tool::{Config, mime};

pub fn page<C: Config>(server: &OpServer<C>, r: &OpRequest<()>) -> Result<OpResponse<C>> {
    r.token.is_auth()?;
    let users = server.user.read()?;
    let user = users.get(&r.token.uid).ok_or(errs::NOT_FOUND_USER)?;

    Ok(OpResponse::Bytes(
        mime::HTML,
        Bytes::from_owner(
            [H - "html lang=fr"
                + [H - "head" + front::HEAD + [H - "title" + "Menu"]]
                + [H - "body "
                    + op::compo::header(true, "Menu")
                    + [H - "main.w" + &user.name + " [" + user.global.as_str() + "]"]]
                + ""]
            .render_page(),
        ),
    ))
}
