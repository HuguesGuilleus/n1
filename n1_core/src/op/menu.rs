use crate::{op::user::Entity, *};
use n1_html::{H, Html};
use n1_tool::Config;

pub async fn page(server: &OpServer<impl Config>, r: OpRequest<()>) -> Result<String> {
    r.token.is_auth()?;
    let entities = server.entities.read()?;
    let entity = entities.get(&r.token.uid).ok_or(errs::NOT_FOUND_USER)?;
    let user = if let Entity::User(user) = entity {
        user
    } else {
        return Err(errs::EXPECT_USER);
    };

    Ok([H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + "Menu"]]
        + [H - "body "
            + op::compo::header(true, "Menu")
            + [H - "main.w" + &user.name + " [" + user.global.as_str() + "]"]]
        + ""]
    .render_page())
}
