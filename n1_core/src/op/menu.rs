use std::collections::BTreeSet;

use crate::{
    OpRequest, Result, front,
    op::{
        EntityAndObjectNullDTO, Lang, OID_ENTITY_BIO, OID_ENTITY_FS, OID_ENTITY_WIKI, Token, compo,
        user::Entity,
    },
};
use n1_html::{H, Html};
use n1_tool::DB;

pub async fn menu_console(_: &impl DB, r: OpRequest<()>) -> Result<String> {
    r.token.check_auth()?;

    // Get list of group' user
    let mut ids: BTreeSet<u32> = BTreeSet::new();
    for item in r.token.access {
        ids.insert(item.id);
    }
    ids.remove(&r.token.uid);
    ids.remove(&0);
    let mut names: Vec<(u32, &str)> = Vec::with_capacity(1 + ids.len());
    let entities = r.common.entities.read().await;
    names.push((
        r.token.uid,
        entities.get(&r.token.uid).map(Entity::name).unwrap_or("?"),
    ));
    for id in ids {
        names.push((id, entities.get(&id).map(Entity::name).unwrap_or("?")));
    }
    names[1..].sort_by_key(|(_, name)| *name);

    // Generate one rights for an entity (self or group's user)
    fn rights<'a>(lang: &Lang, token: &Token, owner: u32, name: &'a str) -> impl Html + 'a {
        [H + ""
            + [H - "h2" + "@" + name + " [" + owner + "]"]
            + [H - "div.fh.gap"
                + token
                    .access_info(owner, OID_ENTITY_BIO)
                    .map(|_| H - "a.bl href=/_bio/" - owner + lang.bio)
                + token.access_info(owner, OID_ENTITY_FS).map(|_| {
                    H - "a.bl href=/_fs_index/" - EntityAndObjectNullDTO { owner, object: 0 }
                        + lang.fs
                })
                + token
                    .access_info(owner, OID_ENTITY_WIKI)
                    .map(|_| H - "a.bl href=/_wiki/" - owner + lang.wiki)
                + ""]
            + ""]
    }

    // Page generation
    let lang = r.common.lang;
    Ok([H - "html lang=" - lang.code
        + [H - "head" + front::HEAD + [H - "title" + lang.common_menu]]
        + [H - "body"
            + [H - "header.fh"
                + [H - "h1.bl" + lang.common_menu + " @" + names[0].1]
                + compo::header_menu_logged(lang)]
            + [H - "main.w"
                + [H + || {
                    names
                        .iter()
                        .map(|(id, name)| rights(lang, &r.token, *id, *name))
                }]
                + r.token.is_admin.then(|| {
                    [H + [H - "h2" + lang.common_administor]
                        + [H - "div.fh.gap" + [H - "a.bl href=_home" + lang.common_home] + ""]]
                })
                + ""]
            + ""]
        + [H - "script" + compo::LOGIN_JS_CONNECTED]
        + ""]
    .render_page())
}
