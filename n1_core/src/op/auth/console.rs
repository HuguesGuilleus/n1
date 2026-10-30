use std::iter::once;

use n1_html::{DirectHTML, H, Html};
use n1_js::js;
use n1_tool::DB;

use crate::{
    OpRequest, Result, front,
    op::{Entity, OID_ENTITY_BIO, User, compo},
};

pub async fn auth_global_admin(_: &impl DB, r: OpRequest<()>) -> Result<String> {
    r.token.check_isadmin()?;

    let entities = r.common.entities.read().await;
    let mut users: Vec<&User> = entities
        .values()
        .filter_map(|entity| match entity {
            Entity::User(user) => Some(user),
            _ => None,
        })
        .collect();
    users.sort_by_key(|user| &user.name);

    let lang = r.common.lang;
    let console = [H - "html lang=" - lang.code
        + [H - "head" + front::HEAD + [H - "title" + lang.auth_admin_global]]
        + [H - "body"
            + compo::goto_main(lang)
            + [H - "header.fh"
                + [H - "h1.bl" + lang.auth_admin_global]
                + compo::header_menu_logged(lang)
                + ""]
            + [H - "main id=main"
                + [H - "table"
                    + [H + || {
                        users.iter().map(|user| {
                            H + [H - "tr"
                                + [H - "td rowspan=" - (user.groups().count() + 1)
                                    + "#"
                                    + user.uid
                                    + " @"
                                    + &user.name]
                                + [H - "td" + ""]
                                + [H - "td"
                                    + || {
                                        once(
                                            user.access
                                                .iter()
                                                .find(|token_item| {
                                                    token_item.id == user.uid
                                                        && token_item.app == OID_ENTITY_BIO
                                                })
                                                .is_some(),
                                        )
                                    }]]
                                + || {
                                    user.groups().map(|group| {
                                        H - "tr"
                                            + [H - "td"
                                                + entities
                                                    .get(&group)
                                                    .unwrap_or(&Entity::None(0))
                                                    .name()]
                                            + [H - "td" + "$"]
                                    })
                                }
                        })
                    }]]]
            + [H - "script" + [H + "const owner=" + true + ","] + DirectHTML(js!(true)) + ""]
            + ""]];

    Ok(console.render_page())
}
