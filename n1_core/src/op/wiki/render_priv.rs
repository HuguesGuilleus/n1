use n1_html::{H, Html, Q};
use n1_tool::{AtomicError, Config, Result};

use crate::{
    OpRequest, OpServer,
    errs::{self},
    front,
    op::{EntityAndObjectDTO, OID_ENTITY_WIKI, compo, user::Entity, wiki::State},
};

pub async fn render_priv_index(
    server: &OpServer<impl Config>,
    r: OpRequest<u32>,
) -> Result<String> {
    r.token.check_access_write(r.dto, OID_ENTITY_WIKI)?;

    let owner = {
        let entities = server.entities.read().map_err(AtomicError::from)?;
        match entities.get(&r.dto) {
            Some(Entity::Group(group)) => group.clone(),
            Some(_) => return Err(errs::FORBIDEN_GROUP.into()),
            None => return Err(errs::NOT_FOUND.into()),
        }
    };
    let wiki: State = server
        .config
        .obj_fetch(r.dto, OID_ENTITY_WIKI as u32)
        .await?;

    Ok([H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + "Wiki"]]
        + [H - "body"
            + compo::header(
                H + [H - "a.bg href=/_wiki/" - owner.gid + "wiki" + " @" + &owner.name]
                    + " "
                    + "Édition du wiki",
            )
            + [H - "main.w.fv.gap"
                + (wiki.shadow == 0).then(|| {
                    [H - "div.bl.fv.gap"
                        + [H - "a.bl href=/@" - &owner.name - "/w/ " + "/@" + &owner.name + "/w/"]
                        + [H - "div" + [H - "button.bl" + "Rendre caché"]]
                        + ""]
                })
                + (wiki.shadow != 0).then(|| {
                    [H - "div.bl.fv.gap"
                        + [H - "a.bl href=/." - wiki.shadow - "/" + "/." + wiki.shadow + "/"]
                        + [H - "div.fh.gap"
                            + [H - "button.bl" + "Regénéré le code"]
                            + [H - "button.bl" + "Rendre caché"]
                            + ""]
                        + ""]
                })
                + [H - "div.fh" + [H - "a.bl href=/_wiki_new/" - owner.gid + "Nouvelle page"]]
                + (|| {
                    wiki.articles.iter().map(|page| {
                        H - "a.bl href=/_wiki_page/"
                            - EntityAndObjectDTO {
                                eid: owner.gid,
                                oid: page.oid,
                            }
                            + [H - "div" + [H - "b" + &page.title]]
                            + [H - "div" + ".../" + page.oid + "-" + &page.slug]
                            + [H - "time" + page.last_edit]
                    })
                })
                + ""]
            + ""]
        + [H - "script" + compo::LOGIN_JS + compo::TIME_JS]
        + [H - ""]
        + ""]
    .render_page())
}

pub async fn render_priv_page(
    server: &OpServer<impl Config>,
    r: OpRequest<EntityAndObjectDTO>,
) -> Result<String> {
    r.token.check_access_write(r.dto.eid, OID_ENTITY_WIKI)?;

    let owner = {
        let entities = server.entities.read().map_err(AtomicError::from)?;
        match entities.get(&r.dto.eid) {
            Some(Entity::Group(group)) => group.clone(),
            Some(_) => return Err(errs::FORBIDEN_GROUP.into()),
            None => return Err(errs::NOT_FOUND.into()),
        }
    };

    let wiki: State = server
        .config
        .obj_fetch(r.dto.eid, OID_ENTITY_WIKI as u32)
        .await?;
    let page = &wiki.articles[wiki
        .articles
        .binary_search_by(|page| page.oid.cmp(&r.dto.oid))
        .map_err(|_| errs::NOT_FOUND)?];
    let content = server.config.fs_get(r.dto.eid, r.dto.oid).await?;

    Ok([H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + "Wiki " + &page.title]]
        + [H - "body"
            + compo::header(H +
                [H - "a.bg href=/_wiki/" - owner.gid + "wiki"+ " @" + &owner.name ]
                + " " + "Modification d'une page")
            + [H - "main.w.fv.gap"
                + [H - "i" + "Modification: " + [H - "time" + page.last_edit]]
                + [H - "div.bl.fv.gap"
                    + [H - "div" + [H - "input.bl placeholder=Titre value=" - Q(&page.title)]]
                    + [H - "div" + [H - "button.bl" + "Modifier le titre"]]
                    + ""]
                + [H - "div.bl.fv.gap"
                    + [H - "p"
                        + "Détermine la fin de l'URL de la page. Vous pouver vous inspirer du titre. Ne sont pas acceptés les espaces et les majuscules."]
                    + [H - "div" + [H - "input.bl placeholder=Slug value=" - Q(&page.slug)]]
                    + [H - "div" + [H - "button.bl" + "Modifier le slug"]]]
                + [H - "div.bl.fv.gap"
                    + [H - "div.bl.act contenteditable=plaintext-only"
                        + str::from_utf8(&content).map_err(|_| errs::DB_DECODE)?]
                    + [H - "div" + [H - "button.bl" + "Envoyer"]]
                    + ""]
                + [H - "div" + [H - "button.bl" + "Supprimer"]]
                + ""]
            + ""]
        + [H - "script" + compo::LOGIN_JS + compo::TIME_JS]
        + ""]
    .render_page())
}

pub async fn render_priv_new(server: &OpServer<impl Config>, r: OpRequest<u32>) -> Result<String> {
    r.token.check_access_write(r.dto, OID_ENTITY_WIKI)?;

    let owner = {
        let entities = server.entities.read().map_err(AtomicError::from)?;
        match entities.get(&r.dto) {
            Some(Entity::Group(group)) => group.clone(),
            Some(_) => return Err(errs::FORBIDEN_GROUP.into()),
            None => return Err(errs::NOT_FOUND.into()),
        }
    };

    Ok([H - "html lang=fr"
        + [H - "head" + front::HEAD + [H - "title" + "Nouvelle page du wiki"]]
        + [H - "body"
            + compo::header(
                H + [H - "a.bg href=/_wiki/" - owner.gid + "wiki" + " @" + &owner.name]
                    + " "
                    + "Nouvelle d'une page",
            )
            + [H - "main.w"
                + [H - "div.bl.fv.gap"
                    + [H - "div" + [H - "input.bl placeholder=Titre"]]
                    + [H - "div" + [H - "input.bl placeholder='Identifiant de l URL'"]]
                    + [H - "div" + [H - "button.bl" + "Nouvelle page"]]
                    + ""]
                + ""]
            + ""]
        + [H - "script" + compo::LOGIN_JS + compo::TIME_JS]
        + ""]
    .render_page())
}
